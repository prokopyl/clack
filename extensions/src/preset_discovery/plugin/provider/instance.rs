use crate::preset_discovery::prelude::*;
use crate::utils::handle_panic;
use clack_plugin::extensions::prelude::PluginWrapperError;
use clack_plugin::prelude::PluginError;
use clap_sys::factory::preset_discovery::*;
use std::ffi::c_char;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::panic::AssertUnwindSafe;

/// A provider instance that is ready to be used by the host.
///
/// This is the type to be returned by [`PresetDiscoveryFactoryImpl::create_provider`].
///
/// See the [`ProviderInstance::new`] function for more information on how to create this type.
pub struct ProviderInstance<'a> {
    inner: Box<clap_preset_discovery_provider>,
    lifetime: PhantomData<&'a clap_preset_discovery_provider_descriptor>,
}

impl<'a> ProviderInstance<'a> {
    /// Creates a new [`ProviderInstance`] from a given [provider implementation](ProviderImpl).
    ///
    /// This also needs a reference to the associated [`ProviderDescriptor`], as well as the
    /// [`IndexerInfo`] handle that was passed to [`PresetDiscoveryFactoryImpl::create_provider`].
    ///
    /// See the [`PresetDiscoveryFactoryImpl::create_provider`] documentation for a usage example.
    pub fn new<P: ProviderImpl<'a>>(
        indexer: IndexerInfo<'a>,
        descriptor: &'a ProviderDescriptor,
        initializer: impl FnOnce(Indexer<'a>) -> Result<P, PluginError> + 'a,
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

/// The actual data type that is behind the clap_preset_discovery_provider.provider_data pointer.
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

    #[allow(clippy::missing_safety_doc)]
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

            let Ok(provider) = initializer.init(instance.indexer_info.to_indexer()) else {
                return None;
            };

            instance.state = ProviderInstanceState::Initialized(provider);

            Some(())
        })
        .is_some()
    }

    #[allow(clippy::missing_safety_doc)]
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

            match wrapper.get_metadata(location, receiver) {
                Ok(()) => Some(()),
                Err(e) => {
                    let e = PluginWrapperError::from(e);
                    let msg = e.format_cstr();
                    let code = e.os_error_code().unwrap_or(0);

                    receiver.on_error(code, Some(&msg));
                    None
                }
            }
        })
        .is_some()
    }

    #[allow(clippy::missing_safety_doc)]
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

    /// # Safety
    ///
    /// provider must be valid and its data must be a valid instance of P
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
    Initialized(P),
    Destroying,
}

trait Initializer<'a, P>: 'a {
    fn init(self: Box<Self>, indexer: Indexer<'a>) -> Result<P, PluginError>;
}

impl<'a, F: 'a, P> Initializer<'a, P> for F
where
    F: FnOnce(Indexer<'a>) -> Result<P, PluginError>,
{
    #[inline]
    fn init(self: Box<Self>, indexer: Indexer<'a>) -> Result<P, PluginError> {
        self(indexer)
    }
}
