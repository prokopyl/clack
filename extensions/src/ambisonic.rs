//! This extension can be used to specify the ambisonic channel mapping ([`AmbisonicConfig`]) used by the plugin.

use crate::audio_ports::AudioPortType;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::ambisonic::*;
use core::fmt;
use std::ffi::{CStr, c_void};

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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AmbisonicConfig {
    inner: clap_ambisonic_config,
}

/// The ambisonic layout of an audio port, consisting of an [`AmbisonicConfig`] and a channel count.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AmbisonicLayout<'a> {
    /// The ambisonic channel mapping used by the plugin.
    pub config: &'a AmbisonicConfig,
    /// The number of channels in this ambisonic format.
    pub channel_count: u32,
}

/// Component ordering for an ambisonic data exchange format.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AmbisonicOrdering {
    /// Furse-Malham channel ordering
    FuMa = CLAP_AMBISONIC_ORDERING_FUMA,
    /// Ambisonic Channel Number (ACN) ordering
    ACN = CLAP_AMBISONIC_ORDERING_ACN,
}

/// Normalization method for an ambisonic data exchange format.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
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
    /// Creates a new [`AmbisonicConfig`] from a given [`AmbisonicOrdering`] and [`AmbisonicNormalization`].
    #[inline]
    pub const fn new(ordering: AmbisonicOrdering, normalization: AmbisonicNormalization) -> Self {
        Self {
            inner: clap_ambisonic_config {
                normalization: normalization.to_raw(),
                ordering: ordering.to_raw(),
            },
        }
    }

    /// Create an [`AmbisonicConfig`] from a raw [`clap_ambisonic_config`] struct.
    #[inline]
    pub fn from_raw(raw: clap_ambisonic_config) -> Self {
        Self { inner: raw }
    }

    /// Convert this [`AmbisonicConfig`] to its raw [`clap_ambisonic_config`] representation.
    #[inline]
    pub const fn as_raw(&self) -> &clap_ambisonic_config {
        &self.inner
    }

    /// Returns the [`AmbisonicOrdering`] of this configuration.
    #[inline]
    pub const fn ordering(&self) -> Option<AmbisonicOrdering> {
        AmbisonicOrdering::from_raw(self.inner.ordering)
    }

    /// Returns the [`AmbisonicNormalization`] of this configuration.
    #[inline]
    pub const fn normalization(&self) -> Option<AmbisonicNormalization> {
        AmbisonicNormalization::from_raw(self.inner.normalization)
    }
}

impl AmbisonicOrdering {
    /// Create an [`AmbisonicOrdering`] from a raw `u32` value.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            CLAP_AMBISONIC_ORDERING_FUMA => Some(AmbisonicOrdering::FuMa),
            CLAP_AMBISONIC_ORDERING_ACN => Some(AmbisonicOrdering::ACN),
            _ => None,
        }
    }

    /// Convert this [`AmbisonicOrdering`] to its raw `u32` representation.
    pub const fn to_raw(self) -> u32 {
        self as _
    }
}

impl AmbisonicNormalization {
    /// Create an [`AmbisonicNormalization`] from a raw `u32` value.
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            CLAP_AMBISONIC_NORMALIZATION_MAXN => Some(AmbisonicNormalization::MaxN),
            CLAP_AMBISONIC_NORMALIZATION_SN3D => Some(AmbisonicNormalization::SN3D),
            CLAP_AMBISONIC_NORMALIZATION_SN2D => Some(AmbisonicNormalization::SN2D),
            CLAP_AMBISONIC_NORMALIZATION_N3D => Some(AmbisonicNormalization::N3D),
            CLAP_AMBISONIC_NORMALIZATION_N2D => Some(AmbisonicNormalization::N2D),
            _ => None,
        }
    }

    /// Convert this [`AmbisonicNormalization`] to its raw `u32` representation.
    pub const fn to_raw(self) -> u32 {
        self as _
    }
}

impl AudioPortType<'static> {
    /// Ambisonic audio port type.
    pub const AMBISONIC: Self = AudioPortType(CLAP_PORT_AMBISONIC);
}

impl fmt::Debug for AmbisonicConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AmbisonicConfig")
            .field("ordering", &self.ordering())
            .field("normalization", &self.normalization())
            .finish()
    }
}

impl Eq for AmbisonicConfig {}

impl PartialEq for AmbisonicConfig {
    fn eq(&self, other: &Self) -> bool {
        self.inner.ordering == other.inner.ordering
            && self.inner.normalization == other.inner.normalization
    }
}

#[cfg(feature = "configurable-audio-ports")]
mod configurable_audio_ports {
    use super::*;
    use crate::configurable_audio_ports::{AudioPortRequestDetails, PortConfigDetails};

    // SAFETY: AudioPortType::AMBISONIC is the identifier for the Ambisonic port type.
    unsafe impl<'a> PortConfigDetails<'a> for AmbisonicLayout<'a> {
        const PORT_TYPE: AudioPortType<'static> = AudioPortType::AMBISONIC;

        fn to_details(&self) -> AudioPortRequestDetails<'a> {
            // SAFETY: AMBISONIC is valid for any channel count, and the type for it is clap_ambisonic_config as per the CLAP spec
            unsafe {
                AudioPortRequestDetails::from_raw(
                    Some(AudioPortType::AMBISONIC),
                    self.channel_count,
                    self.config.as_raw() as *const clap_ambisonic_config as *const c_void,
                )
            }
        }

        unsafe fn from_details(details: AudioPortRequestDetails<'a>) -> Self {
            // SAFETY: Caller guarantees raw_details is valid matches CLAP_PORT_AMBISONIC,
            // which ensures the details pointer is of type clap_ambisonic_config as per the CLAP spec,
            // which is ABI-compatible with AmbisonicConfig
            unsafe {
                Self {
                    channel_count: details.channel_count(),
                    config: &*details.raw_details().cast::<AmbisonicConfig>(),
                }
            }
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
