#![deny(missing_docs)]

//! Loading and handling of CLAP plugin bundle files.
//!
//! CLAP plugins are distributed in binary files called bundles, which are prebuilt
//! dynamically-loaded libraries (usually `.dll` or `.so` files) with a `.clap` extension.
//! They expose a single [`EntryDescriptor`], which, once initialized, acts as the entry
//! point for the host to read into the bundle.
//!
//! CLAP plugin bundles expose implementations of various standard [factories](FactoryPointer), which are
//! singletons implementing various functionalities. The most relevant is the [`PluginFactory`],
//! which allows to list and instantiate plugins. See the [`factory`](crate::factory) module
//! documentation to learn more about factories.
//!
//! Clack handles all of this functionality through the [`PluginBundle`] type, which exposes all
//! the bundle's factory implementation, and allow the bundle to be loaded in two different ways:
//!
//! * From a file, using [`PluginBundle::load`].
//!
//!   This is the most common usage, as it allows to load
//!   third-party CLAP bundles present anywhere on the file system, which is most likely the
//!   functionality "CLAP plugin support" implies for most hosts.
//!
//! * From a static [`EntryDescriptor`] reference, using [`PluginBundle::load_from_raw`].
//!   
//!   This is a more advanced usage, and it allows to load plugins that have been statically built
//!   into the host's binary (i.e. built-in plugins) without having to distribute them in separate
//!   files, or to perform any filesystem access or plugin discovery.
//!
//!   If needed, this also allows host implementations to not use Clack's implementation of bundle
//!   loading (which uses [`libloading`](https://crates.io/crates/libloading) under the hood), and
//!   implement their own instead.
//!
//! See the [`PluginBundle`]'s type documentation for examples.
//!
//! # Plugin bundle discovery
//!
//! As of now, Clack does not implement any utilities to aid host implementations with discovering
//! which CLAP bundle files are available to be loaded on the filesystem.
//!
//! Refer to the
//! [CLAP specification](https://github.com/free-audio/clap/blob/main/include/clap/entry.h) for more
//! information about standard search paths and the general discovery process.

use std::error::Error;
use std::ffi::NulError;
use std::fmt::{Display, Formatter};

use std::ptr::NonNull;

mod cache;
mod entry;

#[cfg(feature = "libloading")]
mod library;

#[cfg(test)]
pub mod diva_stub;

use crate::bundle::cache::CachedEntry;
use crate::factory::{FactoryPointer, PluginFactory};
pub use clack_common::entry::*;
use clack_common::utils::ClapVersion;

/// A handle to a loaded CLAP plugin bundle file.
///
/// This allows getting all of the [factories](FactoryPointer) exposed by the bundle, mainly the
/// [`PluginFactory`] which allows to list plugin instances.
///
/// This is only a lightweight handle: plugin bundles are only loaded once, and the [`Clone`]
/// operation on this type only clones the handle.
///
/// Plugin bundles are only unloaded when all handles are dropped and all associated instances
/// are unloaded.
///
/// A [`PluginBundle`] can also be loaded from a static [`EntryDescriptor`] instead of a file.
/// See [`PluginBundle::load_from_raw`].
///
/// See the [module docs](crate::bundle) for more information about CLAP bundles.
///
/// # Example
///
/// ```no_run
/// # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use clack_host::prelude::PluginBundle;
///
/// let bundle = PluginBundle::load("/home/user/.clap/u-he/libdiva.so")?;
/// let plugin_factory = bundle.get_plugin_factory().unwrap();
///
/// println!("Loaded bundle CLAP version: {}", bundle.version());
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct PluginBundle {
    inner: CachedEntry,
}

impl PluginBundle {
    /// Loads a CLAP bundle from a file located a the given path.
    ///
    /// # Errors
    ///
    /// This method returns an error if loading the bundle fails.
    /// See [`PluginBundleError`] for all the possible errors that may occur.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use clack_host::prelude::PluginBundle;
    ///
    /// let bundle = PluginBundle::load("/home/user/.clap/u-he/libdiva.so")?;
    ///
    /// println!("Loaded bundle CLAP version: {}", bundle.version());
    /// # Ok(()) }
    /// ```
    #[cfg(feature = "libloading")]
    pub fn load<P: AsRef<std::ffi::OsStr>>(path: P) -> Result<Self, PluginBundleError> {
        use crate::bundle::library::PluginEntryLibrary;

        let path = path.as_ref();
        let path_str = path.to_str().ok_or(PluginBundleError::InvalidUtf8Path)?;

        // SAFETY: TODO: make this function actually unsafe
        let library = unsafe { PluginEntryLibrary::load(path)? };

        // SAFETY: TODO
        let inner = unsafe { cache::load_from_library(library, path_str)? };

        Ok(Self { inner })
    }

    /// Loads a CLAP bundle from a `'static` [`EntryDescriptor`].
    ///
    /// Note that CLAP plugins loaded this way still need a valid path, as they may perform various
    /// filesystem operations relative to their bundle files.
    ///
    /// # Errors
    ///
    /// This method returns an error if initializing the entry fails.
    /// See [`PluginBundleError`] for all the possible errors that may occur.
    ///
    /// # Safety
    ///
    /// Users of this function *must* also ensure the [`EntryDescriptor`]'s fields are all
    /// valid as per the
    /// [CLAP specification](https://github.com/free-audio/clap/blob/main/include/clap/entry.h), as
    /// any undefined behavior may otherwise occur.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clack_host::bundle::EntryDescriptor;
    /// use clack_host::prelude::PluginBundle;
    /// # pub fn foo(descriptor: &'static EntryDescriptor) -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let descriptor: &'static EntryDescriptor = /* ... */
    /// # descriptor;
    ///
    /// let path = "/home/user/.clap/u-he/libdiva.so";
    /// let bundle = unsafe { PluginBundle::load_from_raw(descriptor, path)? };
    ///
    /// println!("Loaded bundle CLAP version: {}", bundle.version());
    /// # Ok(()) }
    #[inline]
    pub unsafe fn load_from_raw(
        inner: &'static EntryDescriptor,
        plugin_path: &str,
    ) -> Result<Self, PluginBundleError> {
        Ok(Self {
            inner: cache::load_from_raw(inner, plugin_path)?,
        })
    }

    /// Gets the raw, C-FFI plugin entry descriptor exposed by this bundle.
    #[inline]
    pub fn raw_entry(&self) -> &EntryDescriptor {
        self.inner.raw_entry()
    }

    /// Returns the [`FactoryPointer`] of type `F` exposed by this bundle, if it exists.
    ///
    /// If this bundle does not expose a factory of the requested type, [`None`] is returned.
    ///
    /// If you are looking to fetch the bundle's [`PluginFactory`], you can also use the
    /// [`get_plugin_factory`](PluginBundle::get_plugin_factory) method, which is just a convenience
    /// wrapper around this method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use clack_host::factory::PluginFactory;
    /// use clack_host::prelude::PluginBundle;
    ///
    /// let bundle = PluginBundle::load("/home/user/.clap/u-he/libdiva.so")?;
    /// let plugin_factory = bundle.get_factory::<PluginFactory>().unwrap();
    ///
    /// println!("Found {} plugins.", plugin_factory.plugin_count());
    /// # Ok(()) }
    /// ```
    pub fn get_factory<'a, F: FactoryPointer<'a>>(&'a self) -> Option<F> {
        // SAFETY: this type ensures the function pointer is valid.
        let ptr = unsafe { self.raw_entry().get_factory?(F::IDENTIFIER.as_ptr()) } as *mut _;
        // SAFETY: pointer was created using F's own identifier.
        NonNull::new(ptr).map(|p| unsafe { F::from_raw(p) })
    }

    /// Returns the [`PluginFactory`] exposed by this bundle, if it exists.
    ///
    /// If this bundle does not expose a [`PluginFactory`], [`None`] is returned.
    ///
    /// This is a convenience method, and is equivalent to calling
    /// [`get_factory`](PluginBundle::get_factory) with a [`PluginFactory`] type parameter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use clack_host::prelude::PluginBundle;
    ///
    /// let bundle = PluginBundle::load("/home/user/.clap/u-he/libdiva.so")?;
    /// let plugin_factory = bundle.get_plugin_factory().unwrap();
    ///
    /// println!("Found {} plugins.", plugin_factory.plugin_count());
    /// # Ok(()) }
    /// ```
    #[inline]
    pub fn get_plugin_factory(&self) -> Option<PluginFactory> {
        self.get_factory()
    }

    /// Returns the CLAP version used by this bundle.
    #[inline]
    pub fn version(&self) -> ClapVersion {
        ClapVersion::from_raw(self.raw_entry().clap_version)
    }
}

/// Errors that can occur while loading a [`PluginBundle`].
///
/// See [`PluginBundle::load`] and [`PluginBundle::load_from_raw`].
#[derive(Debug)]
pub enum PluginBundleError {
    /// The path given to [`PluginBundle::load`] is not valid UTF-8.
    InvalidUtf8Path,
    /// The dynamic library file could not be loaded.
    ///
    /// This contains the error type from the underlying
    /// [`libloading`](https://crates.io/crates/libloading) library.
    #[cfg(feature = "libloading")]
    LibraryLoadingError(libloading::Error),
    /// The entry pointer exposed by the dynamic library file is `null`.
    NullEntryPointer,
    /// The exposed entry used an incompatible CLAP version.
    IncompatibleClapVersion {
        /// The CLAP version that the entry uses.
        ///
        /// See [`ClapVersion::CURRENT`] to get the current clap version.
        plugin_version: ClapVersion,
    },
    /// The given path is not a valid C string.
    InvalidNulPath(NulError),
    /// The entry's `init` method failed.
    EntryInitFailed,
}

impl Error for PluginBundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PluginBundleError::InvalidNulPath(e) => Some(e),
            #[cfg(feature = "libloading")]
            PluginBundleError::LibraryLoadingError(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for PluginBundleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginBundleError::EntryInitFailed => f.write_str("Plugin entry initialization failed"),
            PluginBundleError::InvalidNulPath(e) => {
                write!(f, "Invalid plugin descriptor path: {e}")
            }
            #[cfg(feature = "libloading")]
            PluginBundleError::LibraryLoadingError(e) => {
                write!(f, "Failed to load plugin descriptor library: {e}")
            }
            PluginBundleError::InvalidUtf8Path => {
                f.write_str("Plugin descriptor path contains invalid UTF-8")
            }
            PluginBundleError::NullEntryPointer => f.write_str("Plugin entry pointer is null"),
            PluginBundleError::IncompatibleClapVersion { plugin_version } => write!(
                f,
                "Incompatible CLAP version: plugin is v{}, host is v{}",
                plugin_version,
                ClapVersion::CURRENT
            ),
        }
    }
}
