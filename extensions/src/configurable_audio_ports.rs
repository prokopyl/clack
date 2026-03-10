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
use std::ffi::c_char;
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
#[repr(C)]
pub struct AudioPortRequest<'a> {
    /// Is this request for an input port?
    pub is_input: bool,
    /// The port index to configure
    pub port_index: u32,
    /// The requested port type and details
    pub details: AudioPortRequestDetails<'a>,
}

impl<'a> AudioPortRequest<'a> {
    /// Convert a slice of `AudioPortsRequest` to a slice of `clap_audio_port_configuration_request`.
    #[inline]
    pub fn slice_as_raw(
        slice: &'a [AudioPortRequest<'a>],
    ) -> &'a [clap_audio_port_configuration_request] {
        // SAFETY: Safe due to #[repr(C)] and matching ABI
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const _, slice.len()) }
    }

    /// Convert a slice of `clap_audio_port_configuration_request` to a slice of `AudioPortsRequest`.
    ///
    /// # Safety
    ///
    /// Caller must guarantee the following:
    ///
    /// * Every value in the given slice must be valid;
    /// * The `port_type` and `port_details` fields must be valid for `'a`;
    /// * The `port_details` and `channel_count` must be valid for the type of port described by the `port_type` field.
    #[inline]
    pub unsafe fn slice_from_raw(
        slice: &'a [clap_audio_port_configuration_request],
    ) -> &'a [AudioPortRequest<'a>] {
        // SAFETY: Safe due to #[repr(C)] and matching ABI
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const _, slice.len()) }
    }
}

impl<'a> fmt::Debug for AudioPortRequest<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioPortsRequest")
            .field("is_input", &self.is_input)
            .field("port_index", &self.port_index)
            .field("port_type", &self.details.port_type())
            .field("channel_count", &self.details.channel_count())
            .finish_non_exhaustive()
    }
}

/// Details for an audio port configuration request.
///
/// It is a combination of the number of channels, the port type, and raw port details.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AudioPortRequestDetails<'a> {
    channel_count: u32,
    port_type: *const c_char,
    port_details: *const c_void,
    phantom: PhantomData<&'a ()>,
}

impl<'a> AudioPortRequestDetails<'a> {
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
        // SAFETY: This type guarantees this pointer is either NULL or valid for `'a`.
        unsafe { AudioPortType::from_raw(self.port_type) }
    }

    /// Raw port details. Valid for lifetime `'a`.
    pub const fn raw_details(&self) -> *const c_void {
        self.port_details
    }

    /// Attempts to cast this [`AudioPortRequestDetails`] into a specific [`PortConfigDetails`].
    ///
    /// This returns `None` if the given port type does not match this [`AudioPortRequestDetails`] represents.
    pub fn details<T: PortConfigDetails<'a>>(&self) -> Option<T> {
        if self.port_type() != Some(T::PORT_TYPE) {
            return None;
        }

        // SAFETY: We just checked above that the port type matches this instance.
        Some(unsafe { T::from_details(self) })
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
            port_type: match port_type {
                None => null(),
                Some(port_type) => port_type.as_raw(),
            },
            channel_count,
            phantom: PhantomData,
        }
    }
}

/// A specific type of port configuration details that can be used as a [`AudioPortRequestDetails`].
///
/// # Safety
///
/// Implementors *MUST* set the [`PORT_TYPE`](Self::PORT_TYPE) constant to the correct value for
/// the specific port type this type represents, in accordance with the CLAP specification.
pub unsafe trait PortConfigDetails<'a>: Sized {
    /// The port type identifier that this type represents.
    const PORT_TYPE: AudioPortType<'static>;

    /// Creates this type of port configuration details from a reference to a generic
    /// [`AudioPortRequestDetails`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `raw` port request details actually match this type
    /// by checking it with the [`PORT_TYPE`](Self::PORT_TYPE) constant.
    /// This function does not perform that check.
    unsafe fn from_details(raw: &AudioPortRequestDetails<'a>) -> Self;
}

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of};

    // This is so cursed, so I just want all the tests
    #[test]
    fn test() {
        assert_eq!(
            size_of::<AudioPortRequest<'static>>(),
            size_of::<clap_audio_port_configuration_request>()
        );

        assert_eq!(
            align_of::<AudioPortRequest>(),
            align_of::<clap_audio_port_configuration_request>()
        );

        let val = &AudioPortRequest {
            details: AudioPortRequestDetails::stereo(),
            is_input: true,
            port_index: 0,
        };

        // SAFETY: should be the same representation
        let raw_val =
            unsafe { &*(val as *const _ as *const clap_audio_port_configuration_request) };

        assert_eq!(
            &raw const val.details.port_type,
            &raw const raw_val.port_type
        );
        assert_eq!(
            &raw const val.details.port_details,
            &raw const raw_val.port_details
        );
        assert_eq!(
            &raw const val.details.channel_count,
            &raw const raw_val.channel_count
        );
    }
}
