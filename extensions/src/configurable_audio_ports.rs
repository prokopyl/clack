#![deny(missing_docs)]

//! This extension lets the host configure the plugin's input and output audio ports.
//! This is a "push" approach to audio ports configuration.

use crate::audio_ports::AudioPortType;
use clack_common::extensions::{Extension, PluginExtensionSide, RawExtension};
use clap_sys::ext::configurable_audio_ports::{
    CLAP_EXT_CONFIGURABLE_AUDIO_PORTS, CLAP_EXT_CONFIGURABLE_AUDIO_PORTS_COMPAT,
    clap_audio_port_configuration_request, clap_plugin_configurable_audio_ports,
};
use core::fmt;
use std::{
    ffi::{CStr, c_void},
    marker::PhantomData,
    os::raw::c_char,
    ptr::null,
};

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

/// A port type and additional information for a configurable audio port request.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum AudioPortsRequestDetails<'a> {
    /// A request for a mono audio port
    Mono,

    /// A request for a stereo audio port
    Stereo,

    /// A request for an untyped port with a set number of channels.
    Untyped {
        /// Requested number of channels for this port.
        channels: u32,
    },

    /// A request for a surround audio port with a set channel layout.
    #[cfg(feature = "surround")]
    Surround {
        /// Requested surround channel layout.
        channels: &'a [crate::surround::SurroundChannel],
    },

    /// A request for an ambisonic audio port with a set ambisonic configuration.
    #[cfg(feature = "ambisonic")]
    Ambisonic {
        /// Requested ambisonic configuration.
        config: crate::ambisonic::AmbisonicConfig,
        /// Requested number of channels.
        channels: u32,
    },

    #[doc(hidden)]
    Unknown {
        /// The raw port type string provided by the host, if any.
        port_type: AudioPortType<'a>,
        /// The requested number of channels for this port.
        channels: u32,
    },
}

impl<'a> AudioPortsRequestDetails<'a> {
    /// Get the requested port type.
    pub fn port_type(&self) -> Option<AudioPortType<'a>> {
        match self {
            AudioPortsRequestDetails::Mono => Some(AudioPortType::MONO),
            AudioPortsRequestDetails::Stereo => Some(AudioPortType::STEREO),
            AudioPortsRequestDetails::Untyped { .. } => None,
            #[cfg(feature = "surround")]
            AudioPortsRequestDetails::Surround { .. } => Some(AudioPortType::SURROUND),
            #[cfg(feature = "ambisonic")]
            AudioPortsRequestDetails::Ambisonic { .. } => Some(AudioPortType::AMBISONIC),
            AudioPortsRequestDetails::Unknown { port_type, .. } => Some(*port_type),
        }
    }

    /// Get the requested channel count.
    pub fn channel_count(&self) -> u32 {
        match self {
            AudioPortsRequestDetails::Mono => 1,
            AudioPortsRequestDetails::Stereo => 2,
            AudioPortsRequestDetails::Untyped { channels, .. } => *channels,
            #[cfg(feature = "surround")]
            AudioPortsRequestDetails::Surround { channels } => {
                channels.len().try_into().unwrap_or(u32::MAX) // is there a better way?
            }
            #[cfg(feature = "ambisonic")]
            AudioPortsRequestDetails::Ambisonic { channels, .. } => *channels,
            AudioPortsRequestDetails::Unknown { channels, .. } => *channels,
        }
    }

    /// # Safety
    /// The user must ensure the provided pointers are valid for the lifetime of `'a`.
    /// Additionally, the `port_details` pointer must point to a valid structure
    /// corresponding to the given `port_type` and `channel_count`.
    unsafe fn from_raw(
        port_type: *const c_char,
        port_details: *const c_void,
        channels: u32,
    ) -> Self {
        // SAFETY: Pointer validity ensured by the caller.
        let port_type = unsafe { AudioPortType::from_raw(port_type) };

        match port_type {
            None => Self::Untyped { channels },
            Some(port_type) if port_type == AudioPortType::MONO && channels == 1 => Self::Mono,
            Some(port_type) if port_type == AudioPortType::STEREO && channels == 2 => Self::Stereo,

            #[cfg(feature = "surround")]
            Some(port_type) if port_type == AudioPortType::SURROUND && !port_details.is_null() => {
                // SAFETY: details pointer validity is ensured by the caller.
                let surround_channels = unsafe {
                    crate::surround::SurroundChannel::from_raw_slice(std::slice::from_raw_parts(
                        port_details as *const u8,
                        channels as usize,
                    ))
                };

                match surround_channels {
                    Some(channels) => Self::Surround { channels },
                    None => Self::Unknown {
                        port_type,
                        channels,
                    },
                }
            }

            #[cfg(feature = "ambisonic")]
            Some(port_type) if port_type == AudioPortType::AMBISONIC && !port_details.is_null() => {
                // SAFETY: Validity ensured by the caller.
                let config = unsafe {
                    crate::ambisonic::AmbisonicConfig::from_raw(*(port_details as *const _))
                };

                match config {
                    Some(config) => Self::Ambisonic { config, channels },
                    None => Self::Unknown {
                        port_type,
                        channels,
                    },
                }
            }

            Some(port_type) => Self::Unknown {
                port_type,
                channels,
            },
        }
    }

    fn raw_details(&self) -> *const c_void {
        match self {
            #[cfg(feature = "surround")]
            AudioPortsRequestDetails::Surround { channels } => {
                crate::surround::SurroundChannel::as_raw_slice(channels).as_ptr() as *const c_void
            }
            #[cfg(feature = "ambisonic")]
            AudioPortsRequestDetails::Ambisonic { config, .. } => {
                config.as_raw() as *const _ as *const c_void
            }
            _ => null(),
        }
    }
}

/// A request to configure a single audio port.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct AudioPortsRequest<'a>(clap_audio_port_configuration_request, PhantomData<&'a ()>);

impl<'a> AudioPortsRequest<'a> {
    /// Create a new audio port configuration request with the given details.
    pub fn new(is_input: bool, port_index: u32, details: AudioPortsRequestDetails<'a>) -> Self {
        if let AudioPortsRequestDetails::Unknown { .. } = details {
            panic!(
                "AudioPortsRequestDetails::Unknown is only for representing unknown requests from the host, and shouldn't be used to create new requests"
            );
        }

        Self(
            clap_audio_port_configuration_request {
                is_input,
                port_index,
                port_type: details
                    .port_type()
                    .map(AudioPortType::as_raw)
                    .unwrap_or(std::ptr::null()),
                channel_count: details.channel_count(),
                port_details: details.raw_details(),
            },
            PhantomData,
        )
    }

    /// Is this request for an input port?
    pub fn is_input(&self) -> bool {
        self.0.is_input
    }

    /// The port index to configure
    pub fn port_index(&self) -> u32 {
        self.0.port_index
    }

    /// Get the details of this request, including the requested port type and channel count.
    pub fn details(&self) -> AudioPortsRequestDetails<'a> {
        // SAFETY: The raw request is expected to be valid for the lifetime of self.
        unsafe {
            AudioPortsRequestDetails::from_raw(
                self.0.port_type,
                self.0.port_details,
                self.0.channel_count,
            )
        }
    }

    /// Convert a slice of `AudioPortsRequest` to a slice of `clap_audio_port_configuration_request`.
    #[inline]
    pub fn as_raw_slice(
        slice: &'a [AudioPortsRequest<'a>],
    ) -> &'a [clap_audio_port_configuration_request] {
        // SAFETY: Safe due to #[repr(transparent)]
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const _, slice.len()) }
    }

    /// Convert a slice of `clap_audio_port_configuration_request` to a slice of `AudioPortsRequest`.
    #[inline]
    pub fn from_raw_slice(
        slice: &'a [clap_audio_port_configuration_request],
    ) -> &'a [AudioPortsRequest<'a>] {
        // SAFETY: Safe due to #[repr(transparent)]
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const _, slice.len()) }
    }
}

impl<'a> fmt::Debug for AudioPortsRequest<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioPortsRequest")
            .field("is_input", &self.is_input())
            .field("port_index", &self.port_index())
            .field("details", &self.details())
            .finish()
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host;
