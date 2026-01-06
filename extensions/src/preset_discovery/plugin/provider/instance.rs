use crate::preset_discovery::plugin::provider::wrapper::ProviderWrapper;
use crate::preset_discovery::prelude::*;
use crate::utils::handle_panic;
use clap_sys::factory::preset_discovery::*;
use std::ffi::c_char;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::panic::AssertUnwindSafe;

pub struct ProviderInstance<'a> {
    inner: Box<clap_preset_discovery_provider>,
    lifetime: PhantomData<&'a clap_preset_discovery_provider_descriptor>,
}

impl<'a> ProviderInstance<'a> {
    pub fn new<P: ProviderImpl<'a>>(
        indexer: IndexerInfo<'a>,
        descriptor: &'a ProviderDescriptor,
        initializer: impl FnOnce(Indexer<'a>) -> P + 'a,
    ) -> Self {
        Self {
            lifetime: PhantomData,
            inner: Box::new(ProviderInstanceData::new_raw(
                descriptor,
                indexer,
                initializer,
            )),
        }
    }

    #[inline]
    pub(crate) fn into_raw(self) -> *mut clap_preset_discovery_provider {
        ManuallyDrop::new(self).inner.as_mut()
    }
}

// In case the instance is dropped by a faulty plugin factory implementation.
impl Drop for ProviderInstance<'_> {
    #[inline]
    fn drop(&mut self) {
        if let Some(destroy) = self.inner.destroy {
            // SAFETY: the 'destroy' fn is valid as it's provided by us directly.
            unsafe { destroy(self.inner.as_ref()) }
        }
    }
}

struct ProviderInstanceData<'a, P> {
    indexer_info: IndexerInfo<'a>,
    state: ProviderInstanceState<'a, P>,
}

impl<'a, P: ProviderImpl<'a>> ProviderInstanceData<'a, P> {
    fn new_raw(
        descriptor: &'a ProviderDescriptor,
        indexer_info: IndexerInfo<'a>,
        initializer: impl Initializer<'a, P>,
    ) -> clap_preset_discovery_provider {
        clap_preset_discovery_provider {
            desc: descriptor.as_raw(),
            provider_data: Box::into_raw(Box::new(ProviderInstanceData {
                // SAFETY: TODO
                indexer_info,
                state: ProviderInstanceState::Uninitialized(Box::new(initializer)),
            }))
            .cast(),
            init: Some(Self::init),
            get_metadata: Some(Self::get_metadata),
            destroy: Some(Self::destroy),
            get_extension: None,
        }
    }

    unsafe extern "C" fn init(provider: *const clap_preset_discovery_provider) -> bool {
        Self::handle(provider, |instance| {
            let ProviderInstanceState::Uninitialized(_) = &instance.state else {
                return None;
            };

            let ProviderInstanceState::Uninitialized(initializer) =
                core::mem::replace(&mut instance.state, ProviderInstanceState::Initializing)
            else {
                unreachable!()
            };

            let provider = initializer.init(instance.indexer_info.to_indexer());

            instance.state =
                ProviderInstanceState::Initialized(ProviderWrapper { inner: provider });

            Some(())
        })
        .is_some()
    }

    unsafe extern "C" fn get_metadata(
        provider: *const clap_preset_discovery_provider,
        location_kind: clap_preset_discovery_location_kind,
        location_path: *const c_char,
        clap_preset_discovery_metadata_receiver: *const clap_preset_discovery_metadata_receiver,
    ) -> bool {
        Self::handle(provider, |instance| {
            let ProviderInstanceState::Initialized(wrapper) = &mut instance.state else {
                return None;
            };

            let location = Location::from_raw(location_kind, location_path)?;

            let receiver = MetadataReceiver::from_raw(clap_preset_discovery_metadata_receiver);

            wrapper.inner.get_metadata(location, receiver);

            Some(())
        })
        .is_some()
    }

    unsafe extern "C" fn destroy(provider: *const clap_preset_discovery_provider) {
        let destroyable = Self::handle(provider, |instance| {
            instance.state = ProviderInstanceState::Destroying;
            Some(())
        })
        .is_some();

        if !destroyable {
            return;
        }

        {
            let Some(provider) = provider.cast_mut().as_mut() else {
                return;
            };

            let provider_data =
                core::mem::replace(&mut provider.provider_data, core::ptr::null_mut())
                    .cast::<Self>();

            if provider_data.is_null() {
                return;
            }

            let _ = handle_panic(AssertUnwindSafe(|| {
                let _ = Box::from_raw(provider_data);
            }));
        }

        if provider.is_null() {
            return;
        }

        let _ = handle_panic(AssertUnwindSafe(|| {
            let _ = Box::from_raw(provider.cast_mut());
        }));
    }

    unsafe fn handle<T>(
        provider: *const clap_preset_discovery_provider,
        handler: impl FnOnce(&mut Self) -> Option<T>,
    ) -> Option<T> {
        if provider.is_null() {
            return None;
        }
        let data = provider.read().provider_data.cast::<Self>();

        handle_panic(AssertUnwindSafe(|| handler(data.as_mut()?))).ok()?
    }
}

enum ProviderInstanceState<'a, P> {
    Uninitialized(Box<dyn Initializer<'a, P>>),
    Initializing,
    Initialized(ProviderWrapper<P>),
    Destroying,
}

trait Initializer<'a, P>: 'a {
    fn init(self: Box<Self>, indexer: Indexer<'a>) -> P;
}

impl<'a, F: 'a, P> Initializer<'a, P> for F
where
    F: FnOnce(Indexer<'a>) -> P,
{
    #[inline]
    fn init(self: Box<Self>, indexer: Indexer<'a>) -> P {
        self(indexer)
    }
}
