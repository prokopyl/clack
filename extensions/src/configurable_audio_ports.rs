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

/// A request to configure a single audio port.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct AudioPortsRequest<'a>(clap_audio_port_configuration_request, PhantomData<&'a ()>);

impl<'a> AudioPortsRequest<'a> {
    /// Create a new request for a given port
    pub fn new(is_input: bool, port_index: u32, details: AudioPortsRequestDetails<'a>) -> Self {
        Self(
            clap_audio_port_configuration_request {
                is_input,
                port_index,
                channel_count: details.channel_count,
                port_details: details.port_details,
                port_type: details
                    .port_type
                    .as_ref()
                    .map(|t| t.as_raw())
                    .unwrap_or(null()),
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

    /// The requested number of channels
    pub fn channel_count(&self) -> u32 {
        self.0.channel_count
    }

    /// The requested port type
    pub fn port_type(&self) -> Option<AudioPortType<'a>> {
        // SAFETY: The raw pointer is expected to be valid for the lifetime of self.
        unsafe { AudioPortType::from_raw(self.0.port_type) }
    }

    /// The requested port details
    pub fn details(&self) -> AudioPortsRequestDetails<'a> {
        AudioPortsRequestDetails {
            channel_count: self.channel_count(),
            port_type: self.port_type(),
            port_details: self.0.port_details,
            phantom: PhantomData,
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
            .field("port_type", &self.port_type())
            .field("channel_count", &self.channel_count())
            .finish_non_exhaustive()
    }
}

/// Details for an audio port configuration request.
///
/// It is a combination of the number of channels, the port type, and raw port details.
pub struct AudioPortsRequestDetails<'a> {
    pub(crate) channel_count: u32,
    pub(crate) port_type: Option<AudioPortType<'a>>,
    pub(crate) port_details: *const std::ffi::c_void,
    pub(crate) phantom: PhantomData<&'a ()>,
}

impl<'a> AudioPortsRequestDetails<'a> {
    /// A request for a mono port.
    pub const fn mono() -> Self {
        // SAFETY: This is a valid combination according to the spec
        unsafe { Self::from_raw(Some(AudioPortType::MONO), 1, null()) }
    }

    /// A request for a stereo port.
    pub const fn stereo() -> Self {
        // SAFETY: This is a valid combination according to the spec
        unsafe { Self::from_raw(Some(AudioPortType::STEREO), 2, null()) }
    }

    /// A request for a `null`-typed with the given number of channels.
    pub const fn untyped(channel_count: u32) -> Self {
        // SAFETY: This is a valid combination according to the spec
        unsafe { Self::from_raw(None, channel_count, null()) }
    }

    /// The number of channels requested.
    pub const fn channel_count(&self) -> u32 {
        self.channel_count
    }

    /// The port type requested, if any.
    pub const fn port_type(&self) -> Option<AudioPortType<'a>> {
        self.port_type
    }

    /// Raw port details. Valid for lifetime `'a`.
    pub const fn as_raw(&self) -> *const c_void {
        self.port_details
    }

    /// Create a new port request from raw components.
    ///
    /// # Safety
    /// The caller must ensure that `port_details` is valid for the lifetime `'a`,
    /// and that the combination of `channel_count`, `port_type`, and `port_details` is valid according to the CLAP specification.
    pub const unsafe fn from_raw(
        port_type: Option<AudioPortType<'a>>,
        channel_count: u32,
        port_details: *const c_void,
    ) -> Self {
        Self {
            port_details,
            port_type,
            channel_count,
            phantom: PhantomData,
        }
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host;
