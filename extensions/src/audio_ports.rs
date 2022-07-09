use crate::utils::{data_from_array_buf, from_bytes_until_nul};
use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtension, PluginExtension};
use clap_sys::ext::audio_ports::*;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

pub use clap_sys::ext::audio_ports::{CLAP_PORT_MONO, CLAP_PORT_STEREO};

#[repr(C)]
pub struct PluginAudioPorts(
    clap_plugin_audio_ports,
    PhantomData<*const clap_plugin_audio_ports>,
);

#[repr(C)]
pub struct HostAudioPorts(
    clap_host_audio_ports,
    PhantomData<*const clap_plugin_audio_ports>,
);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct AudioPortType(pub &'static CStr);

pub const MONO_PORT_TYPE: AudioPortType =
    AudioPortType(unsafe { CStr::from_bytes_with_nul_unchecked(b"mono\0") });
pub const STEREO_PORT_TYPE: AudioPortType =
    AudioPortType(unsafe { CStr::from_bytes_with_nul_unchecked(b"stereo\0") });

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
    type ExtensionType = PluginExtension;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginAudioPorts {}
unsafe impl Sync for PluginAudioPorts {}

unsafe impl Extension for HostAudioPorts {
    const IDENTIFIER: &'static CStr = CLAP_EXT_AUDIO_PORTS;
    type ExtensionType = HostExtension;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostAudioPorts {}
unsafe impl Sync for HostAudioPorts {}

#[derive(Clone)]
pub struct AudioPortInfoBuffer {
    inner: MaybeUninit<clap_audio_port_info>,
}

impl Default for AudioPortInfoBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPortInfoBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
        }
    }
}

pub struct AudioPortInfoData<'a> {
    pub id: u32, // TODO: ClapId
    pub name: &'a CStr,
    pub channel_count: u32,
    pub flags: AudioPortFlags,
    pub port_type: Option<AudioPortType>,
    pub in_place_pair: u32,
}

impl<'a> AudioPortInfoData<'a> {
    unsafe fn try_from_raw(raw: &'a clap_audio_port_info) -> Result<Self, ()> {
        Ok(Self {
            id: raw.id,
            name: from_bytes_until_nul(data_from_array_buf(&raw.name))?,
            channel_count: raw.channel_count,
            flags: AudioPortFlags { bits: raw.flags },
            port_type: NonNull::new(raw.port_type as *mut _)
                .map(|ptr| AudioPortType(CStr::from_ptr(ptr.as_ptr())))
                .filter(|t| !t.0.to_bytes().is_empty()),
            in_place_pair: raw.in_place_pair,
        })
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
