use crate::preset_discovery::indexer::{IndexerImpl, IndexerWrapper, RawIndexerDescriptor};
use clack_host::prelude::{HostInfo, PluginEntry};
use clap_sys::factory::preset_discovery::clap_preset_discovery_provider;
use std::ffi::CStr;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;

mod error;
use crate::preset_discovery::host::metadata_receiver::{MetadataReceiverImpl, to_raw};
use crate::preset_discovery::prelude::*;
pub use error::*;

/// A handle to a provider instance.
///
/// This provider is tied to an [indexer](IndexerImpl) of type `I`, its callbacks able to be called
/// by the provider.
///
/// The role of a provider is to first declare some global, indexing-related data to the indexer
/// during initialization (which is done automatically within [`instantiate`](Provider::instantiate)).
/// It can then be queried about the preset metadata at a given [`Location`] using,
/// [`get_metadata`](Provider::get_metadata).
///
///
/// The indexer instance can be directly accessed using the [`indexer`](Provider::indexer) and
/// [`indexer_mut`](Provider::indexer_mut) methods.
///
/// # Example
///
/// ```
/// use std::error::Error;
/// use clack_host::prelude::*;
/// use clack_extensions::preset_discovery::prelude::*;
///
/// fn get_all_metadata(bundle: &PluginEntry, host_info: &HostInfo) -> Result<(), Box<dyn Error>> {
///     let Some(preset_discovery_factory) = bundle.get_factory::<PresetDiscoveryFactory>() else {
///         return Ok(())
///     };
///
///     // This will contain metadata about all of our presets.
///     let mut metadata_receiver = MyMetadataReceiver;
///
///     for provider_descriptor in preset_discovery_factory.provider_descriptors() {
///         let Some(provider_id) = provider_descriptor.id() else {
///             continue
///         };
///
///         let mut provider = Provider::instantiate(MyIndexer, bundle, provider_id, host_info)?;
///         // Retreive location info from the indexer, and discover all files with e.g. walkdir
///         let locations = /* ... */
/// # [Location::Plugin];
///
///         for location in locations {
///             provider.get_metadata(location, &mut metadata_receiver);
///         }
///     }
///
///     // We now have all the presets data in our metadata_receiver, we can do anything we want with them.
///
///     Ok(())
/// }
///
/// struct MyIndexer;
///
/// impl IndexerImpl for MyIndexer {
/// /* ... */
/// # fn declare_filetype(&mut self, _: FileType) -> Result<(), HostError> { Ok(()) }
/// # fn declare_location(&mut self, _: LocationInfo)  -> Result<(), HostError> { Ok(()) }
/// # fn declare_soundpack(&mut self, _: Soundpack)  -> Result<(), HostError> { Ok(()) }
/// }
///
/// struct MyMetadataReceiver;
///
/// impl MetadataReceiverImpl for MyMetadataReceiver {
/// /* ... */
/// # fn on_error(&mut self, _: i32, _: Option<&core::ffi::CStr>) {}
/// # fn begin_preset(&mut self, _: Option<&core::ffi::CStr>, _: Option<&core::ffi::CStr>) -> Result<(), HostError> {Ok(())}
/// # fn add_plugin_id(&mut self, _: UniversalPluginId<'_>) {}
/// # fn set_soundpack_id(&mut self, _: &core::ffi::CStr) {}
/// # fn set_flags(&mut self, _: Flags) {}
/// # fn add_creator(&mut self, _: &core::ffi::CStr) {}
/// # fn set_description(&mut self, _: &core::ffi::CStr) {}
/// # fn set_timestamps(&mut self, _: Option<Timestamp>, _: Option<Timestamp>) {}
/// # fn add_feature(&mut self, _: &core::ffi::CStr) {}
/// # fn add_extra_info(&mut self, _: &core::ffi::CStr, _: &core::ffi::CStr) {}
/// }
/// ```
pub struct Provider<I> {
    indexer_wrapper: Pin<Box<IndexerWrapper<I>>>,
    provider_ptr: NonNull<clap_preset_discovery_provider>,

    // This is only here to be kept alive
    _indexer_descriptor: Pin<Box<RawIndexerDescriptor>>,
    _plugin_bundle: PluginEntry,
    _no_send: PhantomData<*const ()>,
}

impl<I> Provider<I> {
    /// Instantiate a new provider, backed by a given [`indexer`](IndexerImpl) instance of type `I`.
    ///
    /// This method requires a reference to the [`PluginEntry`] that contains the provider, as well
    /// as the unique `provider_id` of the provider to create (since bundles can have multiple providers).
    /// The `provider_id` should come from a [`ProviderDescriptor`], provided by a [`PresetDiscoveryFactory`] instance.
    ///
    /// Moreover, a [`HostInfo`] providing metadata about the host must also be provider.
    ///
    /// See the [module docs](crate::preset_discovery) for a usage example.
    pub fn instantiate(
        indexer: I,
        plugin_entry: &PluginEntry,
        provider_id: &CStr,
        host_info: &HostInfo,
    ) -> Result<Self, ProviderInstanceError>
    where
        I: IndexerImpl,
    {
        let factory: PresetDiscoveryFactory = plugin_entry
            .get_factory()
            .ok_or(ProviderInstanceError::MissingPresetDiscoveryFactory)?;

        let mut indexer_wrapper = IndexerWrapper::new(indexer);
        let mut indexer_descriptor =
            RawIndexerDescriptor::new::<I>(host_info.clone(), indexer_wrapper.as_mut());

        let provider_ptr = create_provider(factory, indexer_descriptor.as_mut(), provider_id)?;

        // SAFETY: The given pointer comes straight from create_provider
        if let Err(e) = unsafe { init_provider(provider_ptr) } {
            // SAFETY: Even though init failed, the provider pointer should still be valid until we call destroy.
            let destroy = unsafe { provider_ptr.read() }.destroy;

            if let Some(destroy) = destroy {
                // SAFETY: It is still safe to call destroy once, since create succeeded.
                unsafe { destroy(provider_ptr.as_ptr()) }
            }

            return Err(e);
        }

        Ok(Self {
            indexer_wrapper,
            _indexer_descriptor: indexer_descriptor,
            provider_ptr,
            _plugin_bundle: plugin_entry.clone(),
            _no_send: PhantomData,
        })
    }

    /// Gets all available preset metadata from a given [`Location`], and sends it to the given
    /// `receiver`.
    ///
    /// The [metadata receiver](MetadataReceiverImpl) internally stores all the metadata that is
    /// received via its callbacks, which can then be directly retrieved.
    #[inline]
    pub fn get_metadata(&mut self, location: Location, receiver: &mut impl MetadataReceiverImpl) {
        let receiver = to_raw(receiver);
        let (location_kind, location_path) = location.to_raw();

        if let Some(get_metadata) = self.raw().get_metadata {
            // SAFETY: This type guarantees provider_ptr is still valid.
            // location_kind and location_path match the expected values from the CLAP spec
            // The receiver pointer comes from a reference, which guarantees it is valid.
            unsafe {
                get_metadata(
                    self.provider_ptr.as_ptr(),
                    location_kind,
                    location_path,
                    &receiver,
                )
            };
        }
    }

    /// Returns the [descriptor](ProviderDescriptor) that is tied to this provider.
    #[inline]
    pub fn descriptor(&self) -> &ProviderDescriptor {
        let desc = self.raw().desc;

        // SAFETY: the descriptor is read-only, and guaranteed by the CLAP spec to still be valid
        // as long as this type is alive.
        let desc = unsafe { &*desc };

        // SAFETY: the CLAP spec guarantees all inner fields are valid.
        unsafe { ProviderDescriptor::from_raw(desc) }
    }

    /// Returns a shared reference to the [indexer](IndexerImpl) instance this provider is tied to.
    #[inline]
    pub fn indexer(&self) -> &I {
        self.indexer_wrapper.inner()
    }

    /// Returns a mutable reference to the [indexer](IndexerImpl) instance this provider is tied to.
    #[inline]
    pub fn indexer_mut(&mut self) -> &mut I {
        self.indexer_wrapper.as_mut().inner_mut()
    }

    /// Returns a pointer to the raw, C-FFI compatible representation of this provider.
    ///
    /// The given pointer is guaranteed to be non-null, and to remain valid as long as this
    /// type is alive.
    #[inline]
    pub fn as_raw(&self) -> *const clap_preset_discovery_provider {
        self.provider_ptr.as_ptr()
    }

    // TODO: get_extension

    fn raw(&self) -> clap_preset_discovery_provider {
        // SAFETY: This type guarantees the provider ptr is valid for reads, until Drop is called.
        unsafe { self.provider_ptr.read() }
    }
}

impl<I> Drop for Provider<I> {
    #[inline]
    fn drop(&mut self) {
        if let Some(destroy) = self.raw().destroy {
            // SAFETY: destroy is always valid to call once, which we control since we are in Drop
            unsafe { destroy(self.provider_ptr.as_ptr()) }
        }
    }
}

impl<I> Debug for Provider<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.descriptor().id();
        let addr = self.provider_ptr;

        match id {
            Some(id) => write!(f, "Provider ({}, {addr:p})", id.to_string_lossy()),
            None => write!(f, "Provider ({addr:p})"),
        }
    }
}

fn create_provider(
    factory: PresetDiscoveryFactory,
    descriptor: Pin<&mut RawIndexerDescriptor>,
    identifier: &CStr,
) -> Result<NonNull<clap_preset_discovery_provider>, ProviderInstanceError> {
    let Some(create) = factory.raw().get().create else {
        return Err(ProviderInstanceError::NullFactoryCreateFunction);
    };

    // SAFETY: The create function comes directly from the same factory pointer, and is guaranteed
    // to be valid by the `PresetDiscoveryFactory` type.
    // The indexer descriptor is correctly filled by ourselves.
    // The identifier is guaranteed to point to valid bytes.
    let provider_ptr = unsafe {
        create(
            factory.raw().as_ptr(),
            descriptor.as_raw_mut(),
            identifier.as_ptr(),
        )
    };

    NonNull::new(provider_ptr.cast_mut()).ok_or(ProviderInstanceError::CreationFailed)
}

/// # Safety
///
/// The given provider_ptr must come straight out of `create_provider`.
///
/// No other methods on it must be called between `create_provider` and this function.
unsafe fn init_provider(
    provider_ptr: NonNull<clap_preset_discovery_provider>,
) -> Result<(), ProviderInstanceError> {
    // SAFETY: The pointer created by create adheres to the CLAP spec and is valid to read
    // until destroy is called.
    let provider = unsafe { provider_ptr.read() };

    let Some(init) = provider.init else {
        return Err(ProviderInstanceError::NullInitFunction);
    };

    // SAFETY: The init function is only called once, and the provider ptr comes straight from create
    let success = unsafe { init(provider_ptr.as_ptr()) };

    if !success {
        return Err(ProviderInstanceError::InitFailed);
    }

    Ok(())
}
