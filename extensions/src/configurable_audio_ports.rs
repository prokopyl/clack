#![deny(missing_docs)]

//! This extension lets the host configure the plugin's input and output audio ports.
//! This is a "push" approach to audio ports configuration.

use crate::audio_ports::AudioPortType;
use clack_common::extensions::{Extension, PluginExtensionSide, RawExtension};
use clap_sys::ext::configurable_audio_ports::{
    CLAP_EXT_CONFIGURABLE_AUDIO_PORTS, CLAP_EXT_CONFIGURABLE_AUDIO_PORTS_COMPAT,
    clap_plugin_configurable_audio_ports,
};
use std::ffi::CStr;

/// The Plugin-side of the Configurable Audio Ports extension.
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
    /// A request for a port of a specific type with no additional information.
    Other(Option<AudioPortType<'a>>),
    // TODO: this enum could be used for future ambisonic/surround implementations (see port_details field in clap_audio_port_configuration_request)
}

impl<'a> AudioPortsRequestPort<'a> {
    /// A request for a mono audio port.
    pub const MONO: Self = AudioPortsRequestPort::Other(Some(AudioPortType::MONO));

    /// A request for a stereo audio port.
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

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;
