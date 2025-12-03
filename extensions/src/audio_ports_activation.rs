use clack_common::extensions::*;
use clap_sys::ext::audio_ports_activation::{
    clap_plugin_audio_ports_activation, CLAP_EXT_AUDIO_PORTS_ACTIVATION,
};
use std::fmt::Display;

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum SampleSize {
    Unspecified = 0,
    Float32 = 32,
    Float64 = 64,
}

impl SampleSize {
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

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginAudioPortsActivation(
    RawExtension<PluginExtensionSide, clap_plugin_audio_ports_activation>,
);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginAudioPortsActivation {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_AUDIO_PORTS_ACTIVATION;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_host::extensions::prelude::*;

    impl PluginAudioPortsActivation {
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

        #[inline]
        pub fn set_active(
            &self,
            plugin: &mut PluginSharedHandle,
            is_input: bool,
            port_index: u32,
            is_active: bool,
            sample_size: u32,
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
                            sample_size,
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

    pub trait PluginAudioPortsActivationImpl {
        fn can_activate_while_processing(&mut self) -> bool;
    }

    // NOTE: we have this trait split b/c if `PluginAudioPortsActivationImpl::can_activate_while_processing` is true,
    // then `set_active` is called from the audio thread (otherwise it is called from the main thread).
    pub trait PluginAudioPortsActivationSharedImpl {
        fn set_active(
            &self,
            is_input: bool,
            port_index: u32,
            is_active: bool,
            sample_size: SampleSize,
        ) -> bool;
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P: Plugin> ExtensionImplementation<P> for PluginAudioPortsActivation
    where
        for<'a> P::Shared<'a>: PluginAudioPortsActivationSharedImpl,
        for<'a> P::MainThread<'a>: PluginAudioPortsActivationImpl,
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
        PluginWrapper::<P>::handle(plugin, |plugin| {
            Ok(plugin
                .main_thread()
                .as_mut()
                .can_activate_while_processing())
        })
        .unwrap_or(false)
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
        for<'a> P::Shared<'a>: PluginAudioPortsActivationSharedImpl,
    {
        let sample_size = SampleSize::from_raw(sample_size).unwrap();
        PluginWrapper::<P>::handle(plugin, |plugin| {
            Ok(plugin
                .shared()
                .set_active(is_input, port_index, is_active, sample_size))
        })
        .unwrap_or(false)
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
