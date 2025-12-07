//! Allows a synthesizer plugin to indicate the number of voices it has.
//!
//! It is useful for the host when performing polyphonic modulations,
//! because the host needs its own voice management and should try to follow
//! what the plugin is doing:
//!
//! * make the host's voice pool coherent with what the plugin has;
//! * turn the host's voice management to mono when the plugin is mono.

#![deny(missing_docs)]

use bitflags::bitflags;
use clack_common::extensions::*;
use clap_sys::ext::voice_info::*;
use std::ffi::CStr;

/// Plugin-side of the Voice Info extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginVoiceInfo(RawExtension<PluginExtensionSide, clap_plugin_voice_info>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginVoiceInfo {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_VOICE_INFO];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// Host-side of the Voice Info extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostVoiceInfo(RawExtension<HostExtensionSide, clap_host_voice_info>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostVoiceInfo {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_VOICE_INFO];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

bitflags! {
    /// Option flags for [`VoiceInfo`].
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct VoiceInfoFlags: u64 {
        /// Allows the host to send overlapping NOTE_On events.
        /// The plugin will then have to rely upon the note_id to distinguish between the different notes.
        const SUPPORTS_OVERLAPPING_NOTES = CLAP_VOICE_INFO_SUPPORTS_OVERLAPPING_NOTES;
    }
}

/// A plugin's voice information.
pub struct VoiceInfo {
    /// The current number of voices the patch can use.
    ///
    /// If this is 1, then the synth is working in mono, and the host can decide to only use global
    /// modulation mapping.
    pub voice_count: u32,
    /// The maximum number of voice the synthesizer can output at the same time.
    pub voice_capacity: u32,
    /// Options for voice information, see [`VoiceInfoFlags`].
    pub flags: VoiceInfoFlags,
}

impl VoiceInfo {
    /// Gets a [`VoiceInfo`] from its raw, C-FFI compatible representation.
    #[inline]
    pub const fn from_raw(raw: &clap_voice_info) -> Self {
        Self {
            voice_count: raw.voice_count,
            voice_capacity: raw.voice_capacity,
            flags: VoiceInfoFlags::from_bits_truncate(raw.flags),
        }
    }

    /// Returns the raw, C-FFI compatible representation of this [`VoiceInfo`].
    #[inline]
    pub const fn to_raw(&self) -> clap_voice_info {
        clap_voice_info {
            voice_count: self.voice_count,
            voice_capacity: self.voice_capacity,
            flags: self.flags.bits(),
        }
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_host::extensions::prelude::*;
    use std::mem::MaybeUninit;

    impl PluginVoiceInfo {
        /// Retrieves a plugin's Voice Information.
        ///
        /// If the plugin failed to provide any Voice Information, this returns [`None`].
        pub fn get(&self, plugin: &mut PluginMainThreadHandle) -> Option<VoiceInfo> {
            let info = MaybeUninit::zeroed();

            // SAFETY: This type ensures the function pointer is valid.
            let success = unsafe {
                plugin.use_extension(&self.0).get?(plugin.as_raw(), info.as_ptr() as *mut _)
            };

            // SAFETY: we only read the buffer if the plugin returned a successful state
            unsafe { success.then(|| VoiceInfo::from_raw(info.assume_init_ref())) }
        }
    }

    /// Implementation of the Host-side of the Voice Info extension.
    pub trait HostVoiceInfoImpl {
        /// Indicates the plugin has changed its voice configuration, and the host needs to update
        /// it by calling [`get`](PluginVoiceInfo::get) again.
        fn changed(&mut self);
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<H> ExtensionImplementation<H> for HostVoiceInfo
    where
        H: for<'a> HostHandlers<MainThread<'a>: HostVoiceInfoImpl>,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_voice_info {
                changed: Some(changed::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn changed<H>(host: *const clap_host)
    where
        H: for<'a> HostHandlers<MainThread<'a>: HostVoiceInfoImpl>,
    {
        HostWrapper::<H>::handle(host, |host| {
            host.main_thread().as_mut().changed();
            Ok(())
        });
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::extensions::prelude::*;

    impl HostVoiceInfo {
        /// Indicates the plugin has changed its voice configuration, and the host needs to update
        /// it by calling [`get`](PluginVoiceInfoImpl::get) again.
        pub fn changed(&self, host: &mut HostMainThreadHandle) {
            if let Some(changed) = host.use_extension(&self.0).changed {
                // SAFETY: This type ensures the function pointer is valid.
                unsafe { changed(host.as_raw()) }
            }
        }
    }

    /// Implementation of the Plugin-side of the Voice Info extension.
    pub trait PluginVoiceInfoImpl {
        /// Retrieves a plugin's Voice Information.
        ///
        /// If the plugin failed to provide any Voice Information, this returns [`None`].
        fn get(&self) -> Option<VoiceInfo>;
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P: Plugin> ExtensionImplementation<P> for PluginVoiceInfo
    where
        for<'a> P: Plugin<MainThread<'a>: PluginVoiceInfoImpl>,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_voice_info {
                get: Some(get::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get<P>(plugin: *const clap_plugin, info: *mut clap_voice_info) -> bool
    where
        for<'a> P: Plugin<MainThread<'a>: PluginVoiceInfoImpl>,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| match plugin.main_thread().as_mut().get() {
            None => Ok(false),
            Some(voice_info) => {
                *info = voice_info.to_raw();
                Ok(true)
            }
        })
        .unwrap_or(false)
    }
}

#[cfg(feature = "clack-plugin")]
pub use plugin::*;
