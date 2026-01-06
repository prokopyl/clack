use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::preset_load::*;
use std::ffi::CStr;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct PluginPresetLoad(RawExtension<PluginExtensionSide, clap_plugin_preset_load>);

// SAFETY: TODO
unsafe impl Extension for PluginPresetLoad {
    const IDENTIFIERS: &'static [&'static CStr] =
        &[CLAP_EXT_PRESET_LOAD, CLAP_EXT_PRESET_LOAD_COMPAT];
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: TODO
        unsafe { Self(raw.cast()) }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct HostPresetLoad(RawExtension<HostExtensionSide, clap_host_preset_load>);

// SAFETY: TODO
unsafe impl Extension for HostPresetLoad {
    const IDENTIFIERS: &'static [&'static CStr] =
        &[CLAP_EXT_PRESET_LOAD, CLAP_EXT_PRESET_LOAD_COMPAT];
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: TODO
        unsafe { Self(raw.cast()) }
    }
}

#[cfg(feature = "clack-host")]
mod host {
    pub(crate) mod indexer;
    pub(crate) mod metadata_receiver;
    pub(crate) mod provider;
    pub use metadata_receiver::MetadataReceiverImpl;
}

#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin {
    pub(crate) mod extension;
    pub(crate) mod indexer;
    pub(crate) mod metadata_receiver;
    pub(crate) mod provider;

    pub use extension::*;
    pub use metadata_receiver::MetadataReceiver;
    pub use provider::{ProviderImpl, ProviderInstance};
}

#[cfg(feature = "clack-plugin")]
pub use plugin::*;

mod descriptor;
pub mod preset_data;

pub mod factory;

pub use descriptor::*;

pub mod indexer {
    #[cfg(feature = "clack-host")]
    pub use super::host::indexer::*;
    #[cfg(feature = "clack-plugin")]
    pub use super::plugin::indexer::*;
}

pub mod provider {
    #[cfg(feature = "clack-host")]
    pub use super::host::provider::*;
    #[cfg(feature = "clack-plugin")]
    pub use super::plugin::provider::*;
}

pub mod prelude {
    pub use super::preset_data::*;
    pub use super::{
        HostPresetLoad, PluginPresetLoad, ProviderDescriptor, factory::PresetDiscoveryFactory,
    };

    #[cfg(feature = "clack-plugin")]
    pub use super::{
        MetadataReceiver, PluginPresetLoadImpl, ProviderImpl, ProviderInstance,
        factory::{PresetDiscoveryFactoryImpl, PresetDiscoveryFactoryWrapper},
        indexer::{Indexer, IndexerInfo},
    };

    #[cfg(feature = "clack-host")]
    pub use super::{MetadataReceiverImpl, indexer::IndexerImpl, provider::Provider};
}
