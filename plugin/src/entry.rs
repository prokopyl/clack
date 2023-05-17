use crate::extensions::wrapper::panic::catch_unwind;
use crate::factory::Factory;
use std::cell::UnsafeCell;
use std::error::Error;
use std::ffi::{c_void, CStr};
use std::fmt::{Display, Formatter};
use std::panic::AssertUnwindSafe;
use std::ptr::NonNull;

pub use clack_common::entry::*;

mod single;

pub use single::SinglePluginEntry;

/// A prelude that's helpful for implementing custom [`Entry`] and [`PluginFactory`](crate::factory::plugin::PluginFactory) types.
pub mod prelude {
    pub use crate::{
        entry::*,
        factory::{
            plugin::{PluginFactory, PluginFactoryWrapper},
            Factory,
        },
        host::HostInfo,
        plugin::{descriptor::PluginDescriptorWrapper, PluginInstance},
    };
}

/// A CLAP bundle's entry point.
///
/// This trait can be implemented by a custom, user-provided type in order to customize the bundle's
/// entrypoint behavior. The [`clack_export_entry!`](crate::clack_export_entry) macro can then be
/// used to set that type as the bundle's entry point to be discovered and loaded by hosts.
///
/// If you only care about the entry exposing a single plugin, you may use the [`SinglePluginEntry`]
/// type instead of making your own.
///
/// To learn more about entries, refer to the [module documentation](self).
///
/// # Example
///
/// The following example shows how to create a custom entry and plugin factory that exposes
/// two different plugins.
///
/// ```
/// use std::ffi::CStr;
/// use clack_plugin::entry::prelude::*;
/// use clack_plugin::prelude::*;
///
/// pub struct MyFirstPlugin;
/// pub struct MySecondPlugin;
///
/// impl Plugin for MyFirstPlugin {
///     /* ... */
/// #   type AudioProcessor<'a> = (); type Shared<'a> = (); type MainThread<'a> = ();
/// #   fn get_descriptor() -> Box<dyn PluginDescriptor> {
/// #       unreachable!()
/// #   }
/// }
///
/// impl Plugin for MySecondPlugin {
///     /* ... */
/// #   type AudioProcessor<'a> = (); type Shared<'a> = (); type MainThread<'a> = ();
/// #   fn get_descriptor() -> Box<dyn PluginDescriptor> {
/// #       unreachable!()
/// #   }
/// }
///
/// pub struct MyEntry {
///     plugin_factory: PluginFactoryWrapper<MyPluginFactory>
/// }
///
/// impl Entry for MyEntry {
///     fn new(bundle_path: &CStr) -> Result<Self, EntryLoadError> {
///         // Initialize the factory and its wrapper
///         Ok(Self { plugin_factory: PluginFactoryWrapper::new(MyPluginFactory::new()) })
///     }
///
///     fn declare_factories<'a>(&'a self, builder: &mut EntryFactories<'a>) {
///         // Expose MyPluginFactory as an available factory, using the wrapper
///         builder.register_factory(&self.plugin_factory);
///     }
/// }
///
/// // Our factory holds the descriptors for both of our plugins
/// pub struct MyPluginFactory {
///     first_plugin: PluginDescriptorWrapper,
///     second_plugin: PluginDescriptorWrapper,
/// }
///
/// impl MyPluginFactory {
///     pub fn new() -> Self {
///         Self {
///             first_plugin: PluginDescriptorWrapper::new(MyFirstPlugin::get_descriptor()),
///             second_plugin: PluginDescriptorWrapper::new(MySecondPlugin::get_descriptor()),
///         }
///     }
/// }
///
/// impl PluginFactory for MyPluginFactory {
///     fn plugin_count(&self) -> u32 {
///         2 // We have 2 plugins to expose to the host
///     }
///
///     // Gets the plugin descriptor matching the given index.
///     // It doesn't matter much which plugin has which index,
///     // but each of our plugins must have an unique one.
///     fn plugin_descriptor(&self, index: u32) -> Option<&PluginDescriptorWrapper> {
///         match index {
///             0 => Some(&self.first_plugin),
///             1 => Some(&self.second_plugin),
///             _ => None,
///         }
///     }
///
///     // Called when the host desires to create a new instance of one of our plugins.
///     // Which plugin it is, is determined by the given plugin_id.
///     fn create_plugin<'a>(
///         &'a self,
///         host_info: HostInfo<'a>,
///         plugin_id: &CStr,
///     ) -> Option<PluginInstance<'a>> {
///         if plugin_id == self.first_plugin.descriptor().id() {
///             // We can use the PluginInstance type to easily make a new instance of a given
///             // plugin type.
///             Some(PluginInstance::new::<MyFirstPlugin>(host_info, &self.first_plugin))
///         } else if plugin_id == self.second_plugin.descriptor().id() {
///             Some(PluginInstance::new::<MySecondPlugin>(host_info, &self.second_plugin))
///         } else {
///             None
///         }
///     }
/// }
///
/// ```
pub trait Entry: Sized + Send + Sync + 'static {
    /// Instantiates the entry.
    ///
    /// The path of the bundle file this entry was loaded from is also given by the host, in case
    /// extra neighboring files need to be loaded.
    ///
    /// # Errors
    ///
    /// This returns [`Err`] if any error occurred during instantiation.
    fn new(bundle_path: &CStr) -> Result<Self, EntryLoadError>;

    /// Declares the factories this entry exposes to the host, by registering them to the given
    /// [`EntryFactories`] builder.
    ///
    /// See the [`EntryFactories`] documentation for more information.
    fn declare_factories<'a>(&'a self, builder: &mut EntryFactories<'a>);
}

/// An error indicating a bundle's entry has failed loading.
///
/// This error is returned by [`Entry::new`] when it fails.
#[derive(Copy, Clone, Debug)]
pub struct EntryLoadError;

impl Display for EntryLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Entry failed to load.")
    }
}

impl Error for EntryLoadError {}

#[macro_export]
macro_rules! clack_export_entry {
    ($entry_type:ty) => {
        #[allow(non_upper_case_globals)]
        #[allow(unsafe_code)]
        #[no_mangle]
        pub static clap_entry: $crate::entry::EntryDescriptor = {
            static HOLDER: $crate::entry::EntryHolder<$entry_type> =
                $crate::entry::EntryHolder::new();

            unsafe extern "C" fn init(plugin_path: *const ::core::ffi::c_char) -> bool {
                HOLDER.init(plugin_path)
            }

            unsafe extern "C" fn deinit() {
                HOLDER.de_init()
            }

            unsafe extern "C" fn get_factory(
                identifier: *const ::core::ffi::c_char,
            ) -> *const ::core::ffi::c_void {
                HOLDER.get_factory(identifier)
            }

            $crate::entry::EntryDescriptor {
                clap_version: $crate::utils::ClapVersion::CURRENT.to_raw(),
                init: Some(init),
                deinit: Some(deinit),
                get_factory: Some(get_factory),
            }
        };
    };
}

/// A lightweight collection of all the factories a given plugin entry supports.
///
/// [`Entry`] implementations are expected to declare the factories they support to the host using
/// the [`register_factory`](EntryFactories::register_factory) method.
pub struct EntryFactories<'a> {
    found: Option<NonNull<c_void>>,
    requested: &'a CStr,
}

impl<'a> EntryFactories<'a> {
    #[inline]
    pub(crate) fn new(requested: &'a CStr) -> Self {
        Self {
            found: None,
            requested,
        }
    }

    #[inline]
    pub(crate) fn found(&self) -> *const c_void {
        self.found
            .map(|p| p.as_ptr())
            .unwrap_or(core::ptr::null_mut())
    }

    /// Adds a given factory implementation to the list of factories this bundle entry supports.
    ///
    /// This method returns the factory itself, allowing for easy method chaining and a builder-like
    /// usage pattern.
    ///
    /// See [`Entry`]'s documentation for an example.
    pub fn register_factory<F: Factory>(&mut self, factory: &'a F) -> &mut Self {
        if self.found.is_some() {
            return self;
        }

        if F::IDENTIFIER == self.requested {
            self.found = Some(factory.get_raw_factory_ptr())
        }

        self
    }
}

#[doc(hidden)]
pub struct EntryHolder<E> {
    inner: UnsafeCell<Option<E>>,
}

// SAFETY: TODO
unsafe impl<E> Send for EntryHolder<E> {}
unsafe impl<E> Sync for EntryHolder<E> {}

#[doc(hidden)]
impl<E: Entry> EntryHolder<E> {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    pub unsafe fn init(&self, plugin_path: *const core::ffi::c_char) -> bool {
        if (*self.inner.get()).is_some() {
            return true;
        }

        let plugin_path = CStr::from_ptr(plugin_path);
        let entry = catch_unwind(|| E::new(plugin_path));

        if let Ok(Ok(entry)) = entry {
            *self.inner.get() = Some(entry);
            true
        } else {
            false
        }
    }

    pub unsafe fn de_init(&self) {
        let _ = catch_unwind(AssertUnwindSafe(|| *self.inner.get() = None));
    }

    pub unsafe fn get_factory(&self, identifier: *const core::ffi::c_char) -> *const c_void {
        if identifier.is_null() {
            return core::ptr::null();
        }

        let Some(entry) = &*self.inner.get() else { return core::ptr::null() };
        let identifier = CStr::from_ptr(identifier);

        catch_unwind(AssertUnwindSafe(|| {
            let mut builder = EntryFactories::new(identifier);
            entry.declare_factories(&mut builder);
            builder.found()
        }))
        .unwrap_or(core::ptr::null())
    }
}
