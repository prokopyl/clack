#![deny(missing_docs)]

//! This extension can be used to specify the ambisonic channel mapping ([`AmbisonicConfig`]) used by the plugin.

use crate::audio_ports::AudioPortType;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::ambisonic::*;
use std::ffi::CStr;

/// The Plugin-side of the Ambisonic extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginAmbisonic(RawExtension<PluginExtensionSide, clap_plugin_ambisonic>);

/// The Host-side of the Ambisonic extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostAmbisonic(RawExtension<HostExtensionSide, clap_host_ambisonic>);

// SAFETY: The type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginAmbisonic {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_AMBISONIC, CLAP_EXT_AMBISONIC_COMPAT];
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: This type is expected to contain a type that is ABI-compatible with the matching extension type.
        Self(unsafe { raw.cast() })
    }
}

// SAFETY: The type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostAmbisonic {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_AMBISONIC, CLAP_EXT_AMBISONIC_COMPAT];
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: This type is expected to contain a type that is ABI-compatible with the matching extension type.
        Self(unsafe { raw.cast() })
    }
}

/// Ambisonic data exchange format for an audio port.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct AmbisonicConfig {
    /// Ambisonic channel ordering
    pub ordering: AmbisonicOrdering,

    /// Ambisonic normalization method
    pub normalization: AmbisonicNormalization,
}

/// Component ordering for an ambisonic data exchange format.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AmbisonicOrdering {
    /// Furse-Malham channel ordering
    FuMa = CLAP_AMBISONIC_ORDERING_FUMA,
    /// Ambisonic Channel Number (ACN) ordering
    ACN = CLAP_AMBISONIC_ORDERING_ACN,
}

/// Normalization method for an ambisonic data exchange format.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AmbisonicNormalization {
    /// maxN normalization scheme
    MaxN = CLAP_AMBISONIC_NORMALIZATION_MAXN,
    /// Schmidt semi-normalisation (3D)
    SN3D = CLAP_AMBISONIC_NORMALIZATION_SN3D,
    /// Schmidt semi-normalisation (2D)
    SN2D = CLAP_AMBISONIC_NORMALIZATION_SN2D,
    /// Full 3D normalization
    N3D = CLAP_AMBISONIC_NORMALIZATION_N3D,
    /// Full 2D normalization
    N2D = CLAP_AMBISONIC_NORMALIZATION_N2D,
}

impl AmbisonicConfig {
    /// Create an [`AmbisonicConfig`] from a raw [`clap_ambisonic_config`] struct.
    ///
    /// Returns [`None`] if the struct contains invalid values for the ordering or normalization fields.
    pub fn from_raw(raw: clap_ambisonic_config) -> Option<Self> {
        Some(Self {
            ordering: AmbisonicOrdering::from_raw(raw.ordering)?,
            normalization: AmbisonicNormalization::from_raw(raw.normalization)?,
        })
    }

    /// Convert this [`AmbisonicConfig`] to its raw [`clap_ambisonic_config`] representation.
    pub const fn as_raw(&self) -> &clap_ambisonic_config {
        // SAFETY: This type is repr(C) and ABI-compatible with clap_ambisonic_config, and all of its fields are valid for the corresponding fields in clap_ambisonic_config.
        unsafe { &*(self as *const Self as *const clap_ambisonic_config) }
    }
}

impl AmbisonicOrdering {
    /// Create an [`AmbisonicOrdering`] from a raw `u32` value.
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            i if i == CLAP_AMBISONIC_ORDERING_FUMA => Some(AmbisonicOrdering::FuMa),
            i if i == CLAP_AMBISONIC_ORDERING_ACN => Some(AmbisonicOrdering::ACN),
            _ => None,
        }
    }

    /// Convert this [`AmbisonicOrdering`] to its raw `u32` representation.
    pub fn to_raw(self) -> u32 {
        self as _
    }
}

impl AmbisonicNormalization {
    /// Create an [`AmbisonicNormalization`] from a raw `u32` value.
    pub fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            i if i == CLAP_AMBISONIC_NORMALIZATION_MAXN => Some(AmbisonicNormalization::MaxN),
            i if i == CLAP_AMBISONIC_NORMALIZATION_SN3D => Some(AmbisonicNormalization::SN3D),
            i if i == CLAP_AMBISONIC_NORMALIZATION_SN2D => Some(AmbisonicNormalization::SN2D),
            i if i == CLAP_AMBISONIC_NORMALIZATION_N3D => Some(AmbisonicNormalization::N3D),
            i if i == CLAP_AMBISONIC_NORMALIZATION_N2D => Some(AmbisonicNormalization::N2D),
            _ => None,
        }
    }

    /// Convert this [`AmbisonicNormalization`] to its raw `u32` representation.
    pub fn to_raw(self) -> u32 {
        self as _
    }
}

impl AudioPortType<'static> {
    /// Ambisonic audio port type.
    pub const AMBISONIC: Self = AudioPortType(CLAP_PORT_AMBISONIC);
}

#[cfg(feature = "configurable-audio-ports")]
impl<'a> crate::configurable_audio_ports::AudioPortsRequestDetails<'a> {
    /// Create a new port request for an ambisonic port with the given configuration
    pub const fn ambisonic(channels: u32, config: &'a AmbisonicConfig) -> Self {
        // SAFETY: The lifetime validity is ensured by the caller
        unsafe {
            Self::from_raw(
                Some(AudioPortType::AMBISONIC),
                channels,
                config.as_raw() as *const _ as *const _,
            )
        }
    }

    /// If this is an ambisonic port, return the ambisonic configuration.
    pub fn as_ambisonic(&self) -> Option<AmbisonicConfig> {
        if self.port_type() == Some(AudioPortType::AMBISONIC) && !self.as_raw().is_null() {
            // SAFETY: According to the spec, if port type is AMBISONIC,
            // then port_details is a valid pointer to a `clap_ambisonic_config` struct
            // https://github.com/free-audio/clap/blob/29ffcc273be7c7c651f6c9953b99e69700e2387a/include/clap/ext/configurable-audio-ports.h#L35
            unsafe { AmbisonicConfig::from_raw(*(self.as_raw() as *const clap_ambisonic_config)) }
        } else {
            None
        }
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
