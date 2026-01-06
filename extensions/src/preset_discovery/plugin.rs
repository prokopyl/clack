use crate::preset_discovery::{PresetDiscoveryFactory, ProviderDescriptor};
use clack_common::factory::Factory;
use clack_plugin::factory::{FactoryImplementation, FactoryWrapper, FactoryWrapperError};
use clap_sys::factory::preset_discovery::*;
use std::ffi::{CStr, c_char};

mod extension;
mod indexer;
mod metadata_receiver;
mod provider;

use crate::utils::cstr_from_nullable_ptr;
pub use extension::*;
pub use indexer::{Indexer, IndexerInfo};
pub use metadata_receiver::MetadataReceiver;
pub use provider::{ProviderImpl, ProviderInstance};

pub struct PresetDiscoveryFactoryWrapper<F> {
    inner: FactoryWrapper<clap_preset_discovery_factory, F>,
}

impl<F: PresetDiscoveryFactoryImpl> PresetDiscoveryFactoryWrapper<F> {
    const RAW: clap_preset_discovery_factory = clap_preset_discovery_factory {
        count: Some(count::<F>),
        get_descriptor: Some(get_descriptor::<F>),
        create: Some(create::<F>),
    };

    pub fn new(inner: F) -> Self {
        Self {
            inner: FactoryWrapper::new(Self::RAW, inner),
        }
    }
}

// TODO: make this impl unsafe
impl<F: PresetDiscoveryFactoryImpl> FactoryImplementation for PresetDiscoveryFactoryWrapper<F> {
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

unsafe extern "C" fn count<F: PresetDiscoveryFactoryImpl>(
    factory: *const clap_preset_discovery_factory,
) -> u32 {
    FactoryWrapper::<_, F>::handle(factory, |factory| Ok(factory.provider_count())).unwrap_or(0)
}

unsafe extern "C" fn get_descriptor<F: PresetDiscoveryFactoryImpl>(
    factory: *const clap_preset_discovery_factory,
    index: u32,
) -> *const clap_preset_discovery_provider_descriptor {
    FactoryWrapper::<_, F>::handle(factory, |factory| {
        match factory.provider_descriptor(index) {
            Some(descriptor) => Ok(descriptor.as_raw() as *const _),
            None => Ok(core::ptr::null()),
        }
    })
    .unwrap_or(core::ptr::null())
}

unsafe extern "C" fn create<F: PresetDiscoveryFactoryImpl>(
    factory: *const clap_preset_discovery_factory,
    indexer: *const clap_preset_discovery_indexer,
    id: *const c_char,
) -> *const clap_preset_discovery_provider {
    FactoryWrapper::<_, F>::handle(factory, |factory| {
        let indexer = IndexerInfo::from_raw(indexer)
            .ok_or(FactoryWrapperError::NulPtr("Invalid indexer pointer"))?;

        let id =
            cstr_from_nullable_ptr(id).ok_or(FactoryWrapperError::NulPtr("Invalid id string"))?;

        let provider = factory.create_provider(indexer, id);

        match provider {
            Some(instance) => Ok(instance.into_raw()),
            None => Ok(core::ptr::null_mut()),
        }
    })
    .unwrap_or(core::ptr::null_mut())
}

pub trait PresetDiscoveryFactoryImpl: Send + Sync {
    /// Returns the number of plugins exposed by this factory.
    fn provider_count(&self) -> u32;

    /// Returns the [`ProviderDescriptor`] of the plugin that is assigned the given index.
    ///
    /// Hosts will usually call this method repeatedly with every index from 0 to the total returned
    /// by [`plugin_count`](Self::provider_count), in order to discover all the plugins
    /// exposed by this factory.
    ///
    /// If the given index is out of bounds, or in general does not match any given plugin, this
    /// returns [`None`].
    fn provider_descriptor(&self, index: u32) -> Option<&ProviderDescriptor>;

    /// Creates a new plugin instance for the plugin type matching the given `plugin_id`.
    ///
    /// If the given `plugin_id` matches against one of the plugin this factory manages,
    /// implementors of this trait then use the [`ProviderInstance::new`] method to instantiate the
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
