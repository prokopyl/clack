use crate::preset_discovery::{PresetDiscoveryFactory, ProviderDescriptor};
use clack_common::factory::Factory;
use clack_plugin::factory::{FactoryImplementation, FactoryWrapper};
use clap_sys::factory::preset_discovery::clap_preset_discovery_factory;
use std::ffi::CStr;

mod indexer;
mod metadata_receiver;
mod provider;

pub use indexer::{Indexer, IndexerInfo};
pub use provider::{Provider, ProviderInstance};

pub struct PresetDiscoveryFactoryWrapper<F> {
    inner: FactoryWrapper<clap_preset_discovery_factory, F>,
}

impl<F> PresetDiscoveryFactoryWrapper<F> {
    const RAW: clap_preset_discovery_factory = clap_preset_discovery_factory {
        count: None,
        get_descriptor: None,
        create: None,
    };

    pub fn new(inner: F) -> Self {
        Self {
            inner: FactoryWrapper::new(Self::RAW, inner),
        }
    }
}

// TODO: make this impl unsafe
impl<F> FactoryImplementation for PresetDiscoveryFactoryWrapper<F> {
    type Factory<'a>
        = PresetDiscoveryFactory<'a>
    where
        Self: 'a;

    type Wrapped = F;

    #[inline]
    fn wrapper(&self) -> &FactoryWrapper<<Self::Factory<'_> as Factory<'_>>::Raw, Self::Wrapped> {
        &self.inner
    }
}

unsafe extern "C" fn count<F>(
    clap_preset_discovery_factory: *const clap_preset_discovery_factory,
) -> u32 {
    todo!()
}

pub trait PresetDiscoveryFactoryImpl: Send + Sync {
    /// Returns the number of plugins exposed by this factory.
    fn provider_count(&self) -> u32;

    /// Returns the [`PluginDescriptor`] of the plugin that is assigned the given index.
    ///
    /// Hosts will usually call this method repeatedly with every index from 0 to the total returned
    /// by [`plugin_count`](PluginFactoryImpl::plugin_count), in order to discover all the plugins
    /// exposed by this factory.
    ///
    /// If the given index is out of bounds, or in general does not match any given plugin, this
    /// returns [`None`].
    fn provider_descriptor(&self, index: u32) -> Option<&ProviderDescriptor>;

    /// Creates a new plugin instance for the plugin type matching the given `plugin_id`.
    ///
    /// If the given `plugin_id` matches against one of the plugin this factory manages,
    /// implementors of this trait then use the [`PluginInstance::new`] method to instantiate the
    /// corresponding plugin implementation.
    ///
    /// If the given `plugin_id` does not match any known plugins to this factory, this method
    /// returns [`None`].
    fn create_provider<'a>(
        &'a self,
        host_info: IndexerInfo<'a>,
        provider_id: &CStr,
    ) -> Option<ProviderInstance<'a>>;
}
