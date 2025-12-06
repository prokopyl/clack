use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clack_common::utils::ClapId;
use clap_sys::ext::audio_ports::*;
use std::ffi::{CStr, c_char};
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginAudioPorts(RawExtension<PluginExtensionSide, clap_plugin_audio_ports>);

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostAudioPorts(RawExtension<HostExtensionSide, clap_host_audio_ports>);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AudioPortType<'a>(pub &'a CStr);

impl AudioPortType<'_> {
    pub const MONO: AudioPortType<'static> = AudioPortType(CLAP_PORT_MONO);
    pub const STEREO: AudioPortType<'static> = AudioPortType(CLAP_PORT_STEREO);

    #[inline]
    pub const fn from_channel_count(channel_count: u32) -> Option<Self> {
        match channel_count {
            1 => Some(Self::MONO),
            2 => Some(Self::STEREO),
            _ => None,
        }
    }

    /// Gets an [`AudioPortType`] from a raw, C-FFI compatible, null-terminated string pointer.
    ///
    /// If the pointer is null, or if the string is empty (i.e. the pointer points to a nul byte),
    /// `None` is returned instead.
    ///
    /// # Safety
    ///
    /// The caller must guarantee the string pointer is valid (see [`CStr::from_ptr`]), unless the
    /// given pointer is null.
    #[inline]
    pub const unsafe fn from_raw(raw: *const c_char) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        // SAFETY: the caller guarantees the pointer is valid *if* it is non-null, which we checked above
        let c_str = unsafe { CStr::from_ptr(raw) };
        if c_str.is_empty() {
            None
        } else {
            Some(AudioPortType(c_str))
        }
    }

    /// Returns this [`AudioPortType`] as a pointer to a raw, C-FFI compatible, null-terminated string pointer.
    ///
    /// The string this pointer points to is at least valid for `'a`.
    #[inline]
    pub const fn as_raw(self) -> *const c_char {
        self.0.as_ptr()
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RescanType: u32 {
        const NAMES = CLAP_AUDIO_PORTS_RESCAN_NAMES;
        const FLAGS = CLAP_AUDIO_PORTS_RESCAN_FLAGS;
        const CHANNEL_COUNT = CLAP_AUDIO_PORTS_RESCAN_CHANNEL_COUNT;
        const PORT_TYPE = CLAP_AUDIO_PORTS_RESCAN_PORT_TYPE;
        const IN_PLACE_PAIR = CLAP_AUDIO_PORTS_RESCAN_IN_PLACE_PAIR;
        const LIST = CLAP_AUDIO_PORTS_RESCAN_LIST;
    }
}

impl RescanType {
    /// Returns `true` if any of the set flag values requires the plugin to be deactivated
    /// before re-scanning.
    /// Otherwise, this returns false.
    ///
    /// As of now, this is true if any flag is set except for [`NAMES`](Self::NAMES).
    #[inline]
    pub const fn requires_deactivate(&self) -> bool {
        const RESTART_REQUIRED: RescanType = RescanType::FLAGS
            .union(RescanType::CHANNEL_COUNT)
            .union(RescanType::PORT_TYPE)
            .union(RescanType::PORT_TYPE)
            .union(RescanType::IN_PLACE_PAIR)
            .union(RescanType::LIST);

        self.intersects(RESTART_REQUIRED)
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AudioPortFlags: u32 {
        const IS_MAIN = CLAP_AUDIO_PORT_IS_MAIN;
        const SUPPORTS_64BITS = CLAP_AUDIO_PORT_SUPPORTS_64BITS;
        const PREFERS_64BITS = CLAP_AUDIO_PORT_PREFERS_64BITS;
        const REQUIRES_COMMON_SAMPLE_SIZE = CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE;
    }
}

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginAudioPorts {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_AUDIO_PORTS];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostAudioPorts {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_AUDIO_PORTS];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AudioPortInfo<'a> {
    pub id: ClapId,
    pub name: &'a [u8],
    pub channel_count: u32,
    pub flags: AudioPortFlags,
    pub port_type: Option<AudioPortType<'a>>,
    pub in_place_pair: Option<ClapId>,
}

impl<'a> AudioPortInfo<'a> {
    /// # Safety
    /// The raw port_type pointer must be a valid C string for the 'a lifetime.
    pub unsafe fn from_raw(raw: &'a clap_audio_port_info) -> Option<Self> {
        use crate::utils::*;

        Some(Self {
            id: ClapId::from_raw(raw.id)?,
            name: data_from_array_buf(&raw.name),
            channel_count: raw.channel_count,
            flags: AudioPortFlags::from_bits_truncate(raw.flags),
            // SAFETY: validity of the pointer is upheld by the caller
            port_type: unsafe { AudioPortType::from_raw(raw.port_type) },

            in_place_pair: ClapId::from_raw(raw.in_place_pair),
        })
    }
}

impl Debug for AudioPortInfo<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioPortInfoData")
            .field("id", &self.id)
            .field("name", &String::from_utf8_lossy(self.name))
            .field("channel_count", &self.channel_count)
            .field("flags", &self.flags)
            .field("port_type", &self.port_type)
            .field("in_place_pair", &self.in_place_pair)
            .finish()
    }
}

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
