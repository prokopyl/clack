use crate::audio_ports::AudioPortType;
use clack_common::extensions::{Extension, PluginExtensionSide, RawExtension};
use clap_sys::ext::configurable_audio_ports::{
    CLAP_EXT_CONFIGURABLE_AUDIO_PORTS, CLAP_EXT_CONFIGURABLE_AUDIO_PORTS_COMPAT,
    clap_audio_port_configuration_request, clap_plugin_configurable_audio_ports,
};
use std::{
    ffi::CStr,
    fmt::{self, Debug},
    ptr::null,
};

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginConfigurableAudioPorts(
    RawExtension<PluginExtensionSide, clap_plugin_configurable_audio_ports>,
);

// SAFETY: The type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginConfigurableAudioPorts {
    const IDENTIFIERS: &[&CStr] = &[
        CLAP_EXT_CONFIGURABLE_AUDIO_PORTS,
        CLAP_EXT_CONFIGURABLE_AUDIO_PORTS_COMPAT,
    ];
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: This type is expected to contain a type that is ABI-compatible with the matching extension type.
        Self(unsafe { raw.cast() })
    }
}

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
/// A port type and additional information for a configurable audio port request.
pub enum AudioPortsRequestPort<'a> {
    Other(Option<AudioPortType<'a>>),
    // TODO: this enum could be used for future ambisonic/surround implementations (see port_details field in clap_audio_port_configuration_request)
}

impl<'a> AudioPortsRequestPort<'a> {
    pub const MONO: Self = AudioPortsRequestPort::Other(Some(AudioPortType::MONO));
    pub const STEREO: Self = AudioPortsRequestPort::Other(Some(AudioPortType::STEREO));

    /// Get the requested port type.
    pub fn port_type(&self) -> Option<AudioPortType<'a>> {
        match self {
            AudioPortsRequestPort::Other(port_type) => *port_type,
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// A request to configure a single audio port.
pub struct AudioPortsRequest<'a> {
    /// Whether this request is for an input or output port.
    pub is_input: bool,

    /// The index of the port to configure.
    pub port_index: u32,

    /// The number of channels requested.
    pub channel_count: u32,

    /// The type of port requested and additional information (if applicable).
    pub port_info: AudioPortsRequestPort<'a>,
}

impl<'a> AudioPortsRequest<'a> {
    /// # Safety
    ///
    /// The user must ensure the provided request is valid for the duration of lifetime `'a`.
    pub unsafe fn from_raw(raw: clap_audio_port_configuration_request) -> Self {
        // SAFETY: the caller ensures the pointer is valid, so we can dereference it here.
        unsafe {
            Self {
                is_input: raw.is_input,
                port_index: raw.port_index,
                channel_count: raw.channel_count,
                port_info: AudioPortsRequestPort::Other(AudioPortType::from_raw(raw.port_type)),
            }
        }
    }

    pub fn into_raw(&self) -> clap_audio_port_configuration_request {
        clap_audio_port_configuration_request {
            is_input: self.is_input,
            port_index: self.port_index,
            channel_count: self.channel_count,
            port_type: self
                .port_info
                .port_type()
                .map(|t| t.0.as_ptr())
                .unwrap_or(null()),
            port_details: null(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct AudioPortsRequestList<'a> {
    raw: &'a [clap_audio_port_configuration_request],
}

impl<'a> AudioPortsRequestList<'a> {
    /// # Safety
    ///
    /// The user must ensure the provided pointer is valid for the duration of lifetime `'a`,
    /// and that it points to an array of at least `len` elements.
    pub unsafe fn from_raw(ptr: *const clap_audio_port_configuration_request, len: u32) -> Self {
        Self {
            // SAFETY: the caller ensures the pointer is valid, so we can create a slice here.
            raw: unsafe { std::slice::from_raw_parts(ptr, len as usize) },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn get(&self, index: usize) -> Option<AudioPortsRequest<'a>> {
        // SAFETY: validity is ensured by the lifetime of self.
        self.raw
            .get(index)
            .map(|r| unsafe { AudioPortsRequest::from_raw(*r) })
    }

    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = AudioPortsRequest<'a>> + DoubleEndedIterator + 'a {
        // SAFETY: validity is ensured by the lifetime of self.
        self.raw
            .iter()
            .map(|r| unsafe { AudioPortsRequest::from_raw(*r) })
    }
}

impl Debug for AudioPortsRequestList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;
