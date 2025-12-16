use crate::preset_discovery::indexer::{Indexer, IndexerWrapper, RawIndexerDescriptor};
use clack_host::prelude::{HostInfo, PluginBundle};
use clap_sys::factory::preset_discovery::clap_preset_discovery_provider;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;

mod error;
use crate::preset_discovery::PresetDiscoveryFactory;
pub use error::*;

pub struct Provider<I> {
    indexer_wrapper: Pin<Box<IndexerWrapper<I>>>,
    indexer_descriptor: Pin<Box<RawIndexerDescriptor>>,
    provider_ptr: NonNull<clap_preset_discovery_provider>,

    _plugin_bundle: PluginBundle,
    _no_send: PhantomData<*const ()>,
}

impl<I: Indexer> Provider<I> {
    pub fn instantiate(
        indexer: impl FnOnce() -> I,
        plugin_bundle: &PluginBundle,
        provider_id: &CStr,
        host_info: HostInfo,
    ) -> Result<Self, ProviderInstanceError> {
        let factory: PresetDiscoveryFactory = plugin_bundle
            .get_factory()
            .ok_or(ProviderInstanceError::MissingPresetDiscoveryFactory)?;

        let mut indexer_wrapper = IndexerWrapper::new(indexer());
        let mut indexer_descriptor =
            RawIndexerDescriptor::new::<I>(host_info, indexer_wrapper.as_mut());

        let provider_ptr = create_provider(factory, indexer_descriptor.as_mut(), provider_id)?;

        // SAFETY: TODO
        unsafe { init_provider(provider_ptr)? };

        Ok(Self {
            indexer_wrapper,
            indexer_descriptor,
            provider_ptr,
            _plugin_bundle: plugin_bundle.clone(),
            _no_send: PhantomData,
        })
    }

    // TODO: get_extension
    // TODO: get_metadata
}

impl<I> Drop for Provider<I> {
    fn drop(&mut self) {
        // SAFETY: TODO
        let provider = unsafe { self.provider_ptr.read() };

        if let Some(destroy) = provider.destroy {
            // SAFETY: TODO
            unsafe { destroy(self.provider_ptr.as_ptr()) }
        }
    }
}

fn create_provider(
    factory: PresetDiscoveryFactory,
    descriptor: Pin<&mut RawIndexerDescriptor>,
    identifier: &CStr,
) -> Result<NonNull<clap_preset_discovery_provider>, ProviderInstanceError> {
    let Some(create) = factory.0.get().create else {
        return Err(ProviderInstanceError::NullFactoryCreateFunction);
    };

    // SAFETY: TODO
    let provider_ptr = unsafe {
        create(
            factory.0.as_ptr(),
            descriptor.as_raw_mut(),
            identifier.as_ptr(),
        )
    };

    NonNull::new(provider_ptr.cast_mut()).ok_or(ProviderInstanceError::CreationFailed)
}

unsafe fn init_provider(
    provider_ptr: NonNull<clap_preset_discovery_provider>,
) -> Result<(), ProviderInstanceError> {
    // SAFETY: TODO
    let provider = unsafe { provider_ptr.read() };

    let Some(init) = provider.init else {
        return Err(ProviderInstanceError::NullInitFunction);
    };

    // SAFETY: TODO
    let success = unsafe { init(provider_ptr.as_ptr()) };

    if !success {
        return Err(ProviderInstanceError::InitFailed);
    }

    Ok(())
}
