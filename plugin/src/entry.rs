//! Types to expose and customize a CLAP bundle's entry.
//!
//! CLAP plugins are distributed in binary files called bundles, which are prebuilt
//! dynamically-loaded libraries (usually `.dll` or `.so` files) with a `.clap` extension.
//! They expose a single [`EntryDescriptor`], which, once initialized, acts as the entry
//! point for the host to read into the bundle.
//!
//! A bundle's [`Entry`] is the only exposed symbol in the library. Once
//! [initialized](Entry::new), its role is to expose a number of [factories](Factory), which are
//! singletons implementing various functionalities. The most relevant is the [`PluginFactory`](crate::factory::plugin::PluginFactory),
//! which allows to list and instantiate plugins. See the [`factory`](crate::factory) module
//! documentation to learn more about factories.
//!
//! An entry type can then be exposed to the host reading the bundle file by using the
//! [`clack_export_entry`](crate::clack_export_entry) macro.
//!
//! See the [`Entry`] trait documentation for information and examples on how to implement your own
//! entry type, or see the provided [`SinglePluginEntry`] convenience type if you only need to
//! expose a single plugin type to the host.

use crate::extensions::wrapper::handle_panic;
use crate::factory::Factory;
use std::error::Error;
use std::ffi::{CStr, c_void};
use std::fmt::{Display, Formatter};
use std::panic::{AssertUnwindSafe, UnwindSafe};
use std::ptr::NonNull;
use std::sync::Mutex;

pub use clack_common::entry::*;

mod single;

pub use single::{DefaultPluginFactory, SinglePluginEntry};

/// A prelude that's helpful for implementing custom [`Entry`] and [`PluginFactory`](crate::factory::plugin::PluginFactory) types.
pub mod prelude {
    pub use crate::{
        entry::{Entry, EntryDescriptor, EntryFactories, EntryLoadError, SinglePluginEntry},
        factory::{
            Factory,
            plugin::{PluginFactory, PluginFactoryWrapper},
        },
        host::HostInfo,
        plugin::{PluginDescriptor, PluginInstance},
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
///     type AudioProcessor<'a> = ();
///     type Shared<'a> = ();
///     type MainThread<'a> = ();
/// }
///
/// impl Plugin for MySecondPlugin {
///     type AudioProcessor<'a> = ();
///     type Shared<'a> = ();
///     type MainThread<'a> = ();
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
///     first_plugin: PluginDescriptor,
///     second_plugin: PluginDescriptor,
/// }
///
/// impl MyPluginFactory {
///     pub fn new() -> Self {
///         Self {
///             first_plugin: PluginDescriptor::new("my.plugin.first", "My first plugin"),
///             second_plugin: PluginDescriptor::new("my.plugin.second", "My second plugin"),
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
///     // but each of our plugins must have a unique one.
///     fn plugin_descriptor(&self, index: u32) -> Option<&PluginDescriptor> {
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
///         if plugin_id == self.first_plugin.id() {
///             // We can use the PluginInstance type to easily make a new instance of a given
///             // plugin type.
///             Some(PluginInstance::new::<MyFirstPlugin>(
///                 host_info,
///                 &self.first_plugin,
///                 |_host| Ok(()) /* Create the shared struct */,
///                 |_host, _shared| Ok(()) /* Create the main thread struct */,
///             ))
///         } else if plugin_id == self.second_plugin.id() {
///             Some(PluginInstance::new::<MySecondPlugin>(
///                 host_info,
///                 &self.second_plugin,
///                 |_host| Ok(()) /* Create the shared struct */,
///                 |_host, _shared| Ok(()) /* Create the main thread struct */,
///             ))
///         } else {
///             None
///         }
///     }
/// }
///
/// clack_export_entry!(MyEntry);
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

/// Exposes a given [`Entry`] type to the host as *the* bundle's entry point.
///
/// This macro exports the standard CLAP symbol `clap_entry`, set to an [`EntryDescriptor`] which
/// relies on the given `entry_type` for behavior.
///
/// Note this means you cannot call this macro twice in the same executable, as the produced symbols
/// will conflict.
///
/// # Example
///
/// This example exposes a custom `MyEntry` entry type. See the [`Entry`] trait documentation for
/// an example of how to actually implement it.
///
/// ```
/// use std::ffi::CStr;
/// use clack_plugin::entry::prelude::*;
/// use clack_plugin::prelude::*;
///
/// pub struct MyEntry;
///
/// impl Entry for MyEntry {
/// #    fn new(bundle_path: &CStr) -> Result<Self, EntryLoadError> {
/// #        unreachable!()
/// #    }
///     /* ... */
/// #    fn declare_factories<'a>(&'a self, builder: &mut EntryFactories<'a>) {
/// #        unreachable!()
/// #    }
/// }
///
/// // The host will now see and use this entry.
/// clack_export_entry!(MyEntry);
/// ```
#[macro_export]
macro_rules! clack_export_entry {
    ($entry_type:ty, $entry_lambda:expr) => {
        #[allow(non_upper_case_globals, missing_docs)]
        #[allow(unsafe_code)]
        #[allow(warnings, unused)]
        #[unsafe(no_mangle)]
        pub static clap_entry: $crate::entry::EntryDescriptor =
            $crate::clack_entry!($entry_type, $entry_lambda);
    };
    ($entry_type:ty) => {
        #[allow(non_upper_case_globals, missing_docs)]
        #[allow(unsafe_code)]
        #[allow(warnings, unused)]
        #[unsafe(no_mangle)]
        pub static clap_entry: $crate::entry::EntryDescriptor = $crate::clack_entry!($entry_type);
    };
}

/// Produces an [`EntryDescriptor`] value from a given [`Entry`] type, but without exposing it.
///
/// This can be useful as an alternative to the usual
/// [`clack_export_entry`](crate::clack_export_entry) macro if you do not want or need to export the
/// given entry, and just need an [`EntryDescriptor`].
#[macro_export]
macro_rules! clack_entry {
    ($entry_type:ty, $entry_lambda:expr) => {
        ({
            #[allow(unsafe_code)]
            const fn _entry() -> $crate::entry::EntryDescriptor {
                static HOLDER: $crate::entry::EntryHolder<$entry_type> =
                    $crate::entry::EntryHolder::new();

                unsafe extern "C" fn init(plugin_path: *const ::core::ffi::c_char) -> bool {
                    HOLDER.init_with(plugin_path, $entry_lambda)
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
            }
            _entry()
        })
    };
    ($entry_type:ty) => {
        ({
            #[allow(unsafe_code)]
            const fn _entry() -> $crate::entry::EntryDescriptor {
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
            }
            _entry()
        })
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
    #[doc(hidden)]
    #[inline]
    pub fn new(requested: &'a CStr) -> Self {
        Self {
            found: None,
            requested,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn found(&self) -> *const c_void {
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

        if F::IDENTIFIERS.contains(&self.requested) {
            self.found = Some(factory.get_raw_factory_ptr())
        }

        self
    }
}

enum EntryHolderInner<E> {
    Initialized { reference_count: usize, entry: E },
    Uninitialized,
}

#[doc(hidden)]
pub struct EntryHolder<E: Entry> {
    inner: Mutex<EntryHolderInner<E>>,
}

use crate::entry::EntryHolderInner::*;

#[doc(hidden)]
impl<E: Entry> EntryHolder<E> {
    #[allow(clippy::new_without_default)] // This is actually a private type
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(Uninitialized),
        }
    }

    /// # Safety
    ///
    /// Users *must* ensure this is called at least once before any other method, and that
    /// `plugin_path` points to a valid, NULL-terminated C string.
    #[inline]
    pub unsafe fn init(&self, plugin_path: *const core::ffi::c_char) -> bool {
        self.init_with(plugin_path, |p| E::new(p))
    }

    /// # Safety
    ///
    /// Users *must* ensure this is called at least once before any other method, and that
    /// `plugin_path` points to a valid, NULL-terminated C string.
    pub unsafe fn init_with(
        &self,
        plugin_path: *const core::ffi::c_char,
        entry_factory: impl FnOnce(&CStr) -> Result<E, EntryLoadError> + UnwindSafe,
    ) -> bool {
        let Ok(Ok(mut inner)) = handle_panic(|| self.inner.lock()) else {
            // A poisoned lock means init() panicked, so we consider the entry unusable.
            // Same if lock() itself panicked.
            return false;
        };

        match &mut *inner {
            Initialized {
                reference_count, ..
            } => {
                *reference_count += 1;
                true
            }
            Uninitialized => {
                let plugin_path = CStr::from_ptr(plugin_path);
                let entry = handle_panic(|| entry_factory(plugin_path));

                if let Ok(Ok(entry)) = entry {
                    *inner = Initialized {
                        entry,
                        reference_count: 1,
                    };
                    true
                } else {
                    false
                }
            }
        }
    }

    /// # Safety
    ///
    /// Users must *only* call this after they are done with the whole entry.
    pub unsafe fn de_init(&self) {
        let Ok(mut inner) = self.inner.lock() else {
            return;
        };

        if let Initialized {
            reference_count, ..
        } = &mut *inner
        {
            if *reference_count > 1 {
                *reference_count -= 1;
            } else {
                let _ = handle_panic(AssertUnwindSafe(|| *inner = Uninitialized));
            }
        }
    }

    /// # Safety
    ///
    /// This must only be called between calls to init and de_init, and identifier must point to a
    /// valid, NULL-terminated C string.
    pub unsafe fn get_factory(&self, identifier: *const core::ffi::c_char) -> *const c_void {
        if identifier.is_null() {
            return core::ptr::null();
        }

        let Ok(inner) = self.inner.lock() else {
            return core::ptr::null();
        };

        let Initialized { entry, .. } = &*inner else {
            return core::ptr::null();
        };

        let identifier = CStr::from_ptr(identifier);

        handle_panic(AssertUnwindSafe(|| {
            let mut builder = EntryFactories::new(identifier);
            entry.declare_factories(&mut builder);
            builder.found()
        }))
        .unwrap_or(core::ptr::null())
    }
}
