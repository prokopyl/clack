//! This extension provides a way for the host to activate and de-activate audio ports.
//! Deactivating a port provides the following benefits:
//! - the plugin knows ahead of time that a given input is not present and can choose
//!   an optimized computation path,
//! - the plugin knows that an output is not consumed by the host, and doesn't need to
//!   compute it.
//!
//! Audio buffers must still be provided if the audio port is deactivated.
//! In such case, they shall be filled with 0 (or whatever is the neutral value in your context)
//! and the constant_mask shall be set.
//!
//! Audio ports are initially in the active state after creating the plugin instance.
//! Audio ports state are not saved in the plugin state, so the host must restore the
//! audio ports state after creating the plugin instance.
//!
//! Audio ports state is invalidated by [`PluginAudioPortsConfig::select`](crate::audio_ports_config::PluginAudioPortsConfig::select) and
//! [`HostAudioPorts::rescan`](crate::audio_ports::HostAudioPorts::rescan) with [`AudioPortRescanFlags::LIST`](crate::audio_ports::AudioPortRescanFlags::LIST).

use clack_common::extensions::*;
use clap_sys::ext::audio_ports_activation::{
    CLAP_EXT_AUDIO_PORTS_ACTIVATION, CLAP_EXT_AUDIO_PORTS_ACTIVATION_COMPAT,
    clap_plugin_audio_ports_activation,
};
use std::ffi::CStr;
use std::fmt::Display;

/// Indicates whether the host will provide 32-bit or 64-bit buffers when processing.
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum SampleSize {
    /// Unspecified/unknown.
    Unspecified = 0,
    /// 32-bit buffers.
    Float32 = 32,
    /// 64-bit buffers.
    Float64 = 64,
}

impl SampleSize {
    /// Gets a [`SampleSize`] from a raw `u32`.
    pub fn from_raw(raw: u32) -> Option<Self> {
        use SampleSize::*;
        match raw {
            0 => Some(Unspecified),
            32 => Some(Float32),
            64 => Some(Float64),
            _ => None,
        }
    }
}

impl Display for SampleSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = match self {
            Self::Unspecified => "Unspecified",
            Self::Float32 => "32-bit",
            Self::Float64 => "64-bit",
        };

        f.write_str(display_str)
    }
}

/// The Plugin-side of the Audio Ports Activation extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginAudioPortsActivation(
    RawExtension<PluginExtensionSide, clap_plugin_audio_ports_activation>,
);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginAudioPortsActivation {
    const IDENTIFIERS: &[&CStr] = &[
        CLAP_EXT_AUDIO_PORTS_ACTIVATION,
        CLAP_EXT_AUDIO_PORTS_ACTIVATION_COMPAT,
    ];
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: This type is expected to contain a type that is ABI-compatible with the matching extension type.
        Self(unsafe { raw.cast() })
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_host::extensions::prelude::*;

    impl PluginAudioPortsActivation {
        /// Returns true if the plugin supports calling [`set_active_audio_active`](Self::set_active_audio_active).
        #[inline]
        pub fn can_activate_while_processing(&self, plugin: &mut PluginMainThreadHandle) -> bool {
            match plugin.use_extension(&self.0).can_activate_while_processing {
                None => false,
                Some(can_activate_while_processing) => {
                    // SAFETY: This type ensures the function pointer is valid.
                    unsafe { can_activate_while_processing(plugin.as_raw()) }
                }
            }
        }

        /// Activate/deactivate the given port while the plugin is deactivated.
        ///
        /// Returns true if the plugin has accepted the change.
        #[inline]
        pub fn set_active_audio_inactive(
            &self,
            plugin: &mut InactivePluginMainThreadHandle,
            is_input: bool,
            port_index: u32,
            is_active: bool,
            sample_size: SampleSize,
        ) -> bool {
            match plugin.use_extension(&self.0).set_active {
                None => false,
                Some(set_active) => {
                    // SAFETY: This type ensures the function pointer is valid.
                    unsafe {
                        set_active(
                            plugin.as_raw(),
                            is_input,
                            port_index,
                            is_active,
                            sample_size as u32,
                        )
                    }
                }
            }
        }

        /// Activate/deactivate the given port while the plugin is active.
        ///
        /// This may be called from the audio thread if [`can_activate_while_processing`](Self::can_activate_while_processing) returns true.
        ///
        /// Returns true if the plugin has accepted the change.
        #[inline]
        pub fn set_active_audio_active(
            &self,
            plugin: &mut PluginAudioProcessorHandle,
            is_input: bool,
            port_index: u32,
            is_active: bool,
            sample_size: SampleSize,
        ) -> bool {
            match plugin.use_extension(&self.0).set_active {
                None => false,
                Some(set_active) => {
                    // SAFETY: This type ensures the function pointer is valid.
                    unsafe {
                        set_active(
                            plugin.as_raw(),
                            is_input,
                            port_index,
                            is_active,
                            sample_size as u32,
                        )
                    }
                }
            }
        }
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::extensions::prelude::*;

    /// Implementation of the Plugin-side of the Audio Ports Activation extension.
    pub trait PluginAudioPortsActivationImpl {
        /// Returns true if the plugin supports calling [`set_active`](PluginAudioPortsActivationSetImpl::set_active) while processing.
        fn can_activate_while_processing(&mut self) -> bool;
    }

    /// Implementation of the Plugin-side of the Audio Ports Activation extension.
    ///
    /// NOTE: we have this trait split b/c if `PluginAudioPortsActivationImpl::can_activate_while_processing` is true,
    /// then `set_active` can be called from the audio thread (otherwise it is called from the main thread).
    pub trait PluginAudioPortsActivationSetImpl {
        /// Activate/deactivate the given port.
        ///
        /// Returns true if the plugin has accepted the change.
        fn set_active(
            &mut self,
            is_input: bool,
            port_index: u32,
            is_active: bool,
            sample_size: SampleSize,
        ) -> bool;
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P: Plugin> ExtensionImplementation<P> for PluginAudioPortsActivation
    where
        for<'a> P::AudioProcessor<'a>: PluginAudioPortsActivationSetImpl,
        for<'a> P::MainThread<'a>:
            PluginAudioPortsActivationImpl + PluginAudioPortsActivationSetImpl,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_audio_ports_activation {
                can_activate_while_processing: Some(can_activate_while_processing::<P>),
                set_active: Some(set_active::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn can_activate_while_processing<P: Plugin>(
        plugin: *const clap_plugin,
    ) -> bool
    where
        for<'a> P::MainThread<'a>: PluginAudioPortsActivationImpl,
    {
        // SAFETY: panics are caught by PluginWrapper so they don't cross FFI boundary
        unsafe {
            PluginWrapper::<P>::handle(plugin, |plugin| {
                Ok(plugin
                    .main_thread()
                    .as_mut()
                    .can_activate_while_processing())
            })
            .unwrap_or(false)
        }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn set_active<P: Plugin>(
        plugin: *const clap_plugin,
        is_input: bool,
        port_index: u32,
        is_active: bool,
        sample_size: u32,
    ) -> bool
    where
        for<'a> P::AudioProcessor<'a>: PluginAudioPortsActivationSetImpl,
        for<'a> P::MainThread<'a>: PluginAudioPortsActivationSetImpl,
    {
        // SAFETY: panics are caught by PluginWrapper so they don't cross FFI boundary
        unsafe {
            PluginWrapper::<P>::handle(plugin, |plugin| {
                let sample_size = SampleSize::from_raw(sample_size)
                    .ok_or(PluginWrapperError::InvalidParameter("sample_size"))?;

                // Handle forwarding to the correct implementation
                match plugin.audio_processor() {
                    Ok(mut audio) => {
                        // audio is active, so this must be done on the audio thread
                        Ok(audio
                            .as_mut()
                            .set_active(is_input, port_index, is_active, sample_size))
                    }
                    Err(PluginWrapperError::DeactivatedPlugin) => {
                        // audio thread is *not* active, so this is to be done on the main thread
                        Ok(plugin.main_thread().as_mut().set_active(
                            is_input,
                            port_index,
                            is_active,
                            sample_size,
                        ))
                    }
                    Err(e) => {
                        // forward any other error
                        Err(e)
                    }
                }
            })
            .unwrap_or(false)
        }
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
