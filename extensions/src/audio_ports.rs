use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide};
use clap_sys::ext::audio_ports::*;
use clap_sys::id::CLAP_INVALID_ID;
use std::ffi::CStr;
use std::marker::PhantomData;

#[repr(C)]
pub struct PluginAudioPorts(
    clap_plugin_audio_ports,
    PhantomData<*const clap_plugin_audio_ports>,
);

#[repr(C)]
pub struct HostAudioPorts(
    clap_host_audio_ports,
    PhantomData<*const clap_host_audio_ports>,
);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AudioPortType<'a>(pub &'a CStr);

impl<'a> AudioPortType<'a> {
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
}

bitflags! {
    #[repr(C)]
    pub struct RescanType: u32 {
        const NAMES = CLAP_AUDIO_PORTS_RESCAN_NAMES;
        const FLAGS = CLAP_AUDIO_PORTS_RESCAN_FLAGS;
        const CHANNEL_COUNT = CLAP_AUDIO_PORTS_RESCAN_CHANNEL_COUNT;
        const PORT_TYPE = CLAP_AUDIO_PORTS_RESCAN_PORT_TYPE;
        const IN_PLACE_PAIR = CLAP_AUDIO_PORTS_RESCAN_IN_PLACE_PAIR;
        const LIST = CLAP_AUDIO_PORTS_RESCAN_LIST;
    }
}

bitflags! {
    #[repr(C)]
    pub struct AudioPortFlags: u32 {
        const IS_MAIN = CLAP_AUDIO_PORT_IS_MAIN;
        const SUPPORTS_64BITS = CLAP_AUDIO_PORT_SUPPORTS_64BITS;
        const PREFERS_64BITS = CLAP_AUDIO_PORT_PREFERS_64BITS;
        const REQUIRES_COMMON_SAMPLE_SIZE = CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE;
    }
}

unsafe impl Extension for PluginAudioPorts {
    const IDENTIFIER: &'static CStr = CLAP_EXT_AUDIO_PORTS;
    type ExtensionSide = PluginExtensionSide;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginAudioPorts {}
unsafe impl Sync for PluginAudioPorts {}

unsafe impl Extension for HostAudioPorts {
    const IDENTIFIER: &'static CStr = CLAP_EXT_AUDIO_PORTS;
    type ExtensionSide = HostExtensionSide;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostAudioPorts {}
unsafe impl Sync for HostAudioPorts {}

pub struct AudioPortInfoData<'a> {
    pub id: u32, // TODO: ClapId
    pub name: &'a [u8],
    pub channel_count: u32,
    pub flags: AudioPortFlags,
    pub port_type: Option<AudioPortType<'a>>,
    pub in_place_pair: Option<u32>,
}

impl<'a> AudioPortInfoData<'a> {
    /// # Safety
    /// The raw port_type pointer must be a valid C string for the 'a lifetime.
    pub unsafe fn from_raw(raw: &'a clap_audio_port_info) -> Self {
        use crate::utils::*;
        use std::ptr::NonNull;

        Self {
            id: raw.id,
            name: data_from_array_buf(&raw.name),
            channel_count: raw.channel_count,
            flags: AudioPortFlags { bits: raw.flags },
            port_type: NonNull::new(raw.port_type as *mut _)
                .map(|ptr| AudioPortType(CStr::from_ptr(ptr.as_ptr())))
                .filter(|t| !t.0.to_bytes().is_empty()),

            in_place_pair: if raw.in_place_pair == CLAP_INVALID_ID {
                None
            } else {
                Some(raw.in_place_pair)
            },
        }
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
