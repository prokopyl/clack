#![warn(missing_docs)]

//! Preset discovery and loading.
//!
//! This module contains two parts:
//!
//! * The [`PresetDiscoveryFactory`](factory::PresetDiscoveryFactory), which allows hosts to index
//!   and extract metadata about all presets a plugin supports;
//! * The [`PluginPresetLoad`] extension (and its [host-side counterpart](HostPresetLoad)), which
//!   allows host to load the indexed presets.
//!
//! Combined, these allow to integrate a plugin's preset management straight into the host, by
//! e.g. getting all of those presets into the host's own preset browser for the user to select
//! directly.

use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::preset_load::*;
use std::ffi::CStr;

/// Plugin-side of the Preset Load extension.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct PluginPresetLoad(RawExtension<PluginExtensionSide, clap_plugin_preset_load>);

// SAFETY: CLAP_EXT_PRESET_LOAD & CLAP_EXT_PRESET_LOAD_COMPAT are the IDs of the clap_plugin_preset_load extension, defined by the CLAP spec
unsafe impl Extension for PluginPresetLoad {
    const IDENTIFIERS: &'static [&'static CStr] =
        &[CLAP_EXT_PRESET_LOAD, CLAP_EXT_PRESET_LOAD_COMPAT];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: Pointer type is upheld by caller
        unsafe { Self(raw.cast()) }
    }
}

/// Host-side of the Preset Load extension.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct HostPresetLoad(RawExtension<HostExtensionSide, clap_host_preset_load>);

// SAFETY: CLAP_EXT_PRESET_LOAD & CLAP_EXT_PRESET_LOAD_COMPAT are the IDs of the clap_host_preset_load extension, defined by the CLAP spec
unsafe impl Extension for HostPresetLoad {
    const IDENTIFIERS: &'static [&'static CStr] =
        &[CLAP_EXT_PRESET_LOAD, CLAP_EXT_PRESET_LOAD_COMPAT];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: Pointer type is upheld by caller
        unsafe { Self(raw.cast()) }
    }
}

#[cfg(feature = "clack-host")]
mod host {
    pub(crate) mod extension;
    pub(crate) mod indexer;
    pub(crate) mod metadata_receiver;
    pub(crate) mod provider;
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    pub(crate) mod extension;
    pub(crate) mod indexer;
    pub(crate) mod metadata_receiver;
    pub(crate) mod provider;
}

#[cfg(feature = "clack-host")]
pub use host::extension::{HostPresetLoadImpl, PresetLoadError};

#[cfg(feature = "clack-plugin")]
pub use plugin::extension::PluginPresetLoadImpl;

mod descriptor;
pub mod preset_data;

pub mod factory;

/// Allows plugins to provide information on how their presets can be indexed by the host.
///
/// This interface allows a [preset provider](provider) to describe global information that
/// relates to multiple or all of the presets it handles.
///
/// It must be passed to the preset provider at instantiation time.
pub mod indexer {
    #[cfg(feature = "clack-host")]
    pub use super::host::indexer::*;
    #[cfg(feature = "clack-plugin")]
    pub use super::plugin::indexer::*;
}

/// A plugin-owned interface that provides a list of presets to the host.
pub mod provider {
    pub use super::descriptor::ProviderDescriptor;
    #[cfg(feature = "clack-host")]
    pub use super::host::provider::*;
    #[cfg(feature = "clack-plugin")]
    pub use super::plugin::provider::*;
}

/// A host-owned generic interface that allows [providers](self::provider) to send structured
/// preset metadata to the host.
pub mod metadata_receiver {
    #[cfg(feature = "clack-host")]
    pub use super::host::metadata_receiver::*;
    #[cfg(feature = "clack-plugin")]
    pub use super::plugin::metadata_receiver::*;
}

/// A helpful prelude re-exporting all the types related to preset discovery and loading implementation.
pub mod prelude {
    pub use super::preset_data::*;
    pub use super::{
        HostPresetLoad, PluginPresetLoad, factory::PresetDiscoveryFactory,
        provider::ProviderDescriptor,
    };
    pub use clack_common::utils::{Timestamp, UniversalPluginId};

    #[cfg(feature = "clack-plugin")]
    pub use super::{
        PluginPresetLoadImpl,
        factory::{PresetDiscoveryFactoryImpl, PresetDiscoveryFactoryWrapper},
        indexer::{Indexer, IndexerInfo},
        metadata_receiver::MetadataReceiver,
        provider::{ProviderImpl, ProviderInstance},
    };

    #[cfg(feature = "clack-host")]
    pub use super::{
        indexer::IndexerImpl,
        metadata_receiver::MetadataReceiverImpl,
        provider::{Provider, ProviderInstanceError},
    };
}
