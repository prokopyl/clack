#![deny(missing_docs)]

//! Loading and handling of CLAP plugin bundle files.
//!
//! CLAP plugins are distributed in binary files called bundles, which are prebuilt
//! dynamically-loaded libraries (usually `.dll` or `.so` files) with a `.clap` extension.
//! They expose a single [`EntryDescriptor`], which, once initialized, acts as the entry
//! point for the host to read into the bundle.
//!
//! CLAP plugin bundles expose implementations of various standard [factories](Factory), which are
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
//!   files, or to perform any filesystem access or plugin finder.
//!
//!   If needed, this also allows host implementations to not use Clack's implementation of bundle
//!   loading (which uses [`libloading`](https://crates.io/crates/libloading) under the hood), and
//!   implement their own instead.
//!
//! See the [`PluginBundle`]'s type documentation for examples.
//!
//! # Safety
//!
//! All functions that produce [`PluginBundle`]s from a CLAP bundle file or pointer are inherently
//! unsafe.
//!
//! Most APIs in this crate operate under the assumption that bundles and plugins are compliant
//! to the CLAP specification, and using a number of those APIs can easily result in
//! Undefined Behavior if operating on non-compliant plugins.
//!
//! Therefore, the safe APIs in this crate are safeguarding the host implementation, not on the
//! plugins it loads. As soon as a plugin is loaded, Undefined Behavior is possible to be triggered,
//! regardless of the host's implementation.
//!
//! Users needing to safeguard against crashes or other kinds of UB from plugins from affecting the
//! rest of their application should consider using additional process isolation techniques.
//!
//! # Plugin bundle finder
//!
//! As of now, Clack does not implement any utilities to aid host implementations with discovering
//! which CLAP bundle files are available to be loaded on the filesystem.
//!
//! Refer to the
//! [CLAP specification](https://github.com/free-audio/clap/blob/main/include/clap/entry.h) for more
//! information about standard search paths and the general finder process.

use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

use std::ptr::NonNull;

mod cache;
mod entry;

#[cfg(feature = "libloading")]
mod library;

#[cfg(feature = "clack-plugin")]
mod clack_plugin;

#[cfg(test)]
#[allow(missing_docs)]
pub mod diva_stub;

use crate::bundle::cache::CachedEntry;
use crate::factory::{Factory, plugin::PluginFactory};
pub use clack_common::entry::*;
use clack_common::factory::RawFactoryPointer;
use clack_common::utils::ClapVersion;

/// A handle to a loaded CLAP plugin bundle file.
///
/// This allows getting all the [factories](Factory) exposed by the bundle, mainly the
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
/// let bundle = unsafe { PluginBundle::load("/home/user/.clap/u-he/libdiva.so")? };
/// let plugin_factory = bundle.get_plugin_factory().unwrap();
///
/// println!("Loaded bundle CLAP version: {}", bundle.version());
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct PluginBundle {
    inner: PluginBundleInner,
}

#[derive(Clone)]
enum PluginBundleInner {
    Cached(CachedEntry),
    #[cfg(feature = "clack-plugin")]
    FromClack(clack_plugin::ClackEntry),
}

impl PluginBundle {
    /// Loads a CLAP bundle from a file located at the given path.
    ///
    /// # Safety
    ///
    /// This function loads an external library object file, which is inherently unsafe, as even
    /// just loading it can trigger any behavior in your application, including Undefined Behavior.
    ///
    /// Additionally, loading a non-compliant CLAP bundle may invalidate safety assumptions other
    /// APIs in this library rely on. See the [module docs](self)'s Safety section for more
    /// information.
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
    /// let bundle = unsafe { PluginBundle::load("/home/user/.clap/u-he/libdiva.so")? };
    ///
    /// println!("Loaded bundle CLAP version: {}", bundle.version());
    /// # Ok(()) }
    /// ```
    #[cfg(feature = "libloading")]
    pub unsafe fn load<P: AsRef<std::ffi::OsStr>>(path: P) -> Result<Self, PluginBundleError> {
        use crate::bundle::library::PluginEntryLibrary;
        use std::ffi::CString;

        let bundle_path = std::path::Path::new(path.as_ref());
        let bundle_path_cstr = CString::new(bundle_path.as_os_str().as_encoded_bytes())?;

        let library_path = if cfg!(target_os = "macos")
            && std::fs::metadata(bundle_path)
                .map(|metadata| metadata.is_dir())
                .unwrap_or_default()
        {
            if let Some(file_stem) = bundle_path.file_stem() {
                &*bundle_path.join("Contents/MacOS").join(file_stem)
            } else {
                bundle_path
            }
        } else {
            bundle_path
        };

        let library = PluginEntryLibrary::load(library_path.as_ref())?;

        let inner = cache::load_from_library(library, &bundle_path_cstr)?;

        Ok(Self {
            inner: PluginBundleInner::Cached(inner),
        })
    }

    /// Loads a CLAP bundle from an [`Entry`](::clack_plugin::entry::Entry) created by [`clack_plugin`](::clack_plugin).
    ///
    /// Note that CLAP plugins loaded this way still need a valid path, as they may perform various
    /// filesystem operations relative to their bundle files.
    ///
    /// Note that unlike other methods to load [`PluginBundle`]s, this method is completely safe, as
    /// it can rely on the safety guarantees provided by [`clack_plugin`](::clack_plugin).
    ///
    /// # Errors
    ///
    /// This method returns an error if initializing the entry fails.
    /// See [`PluginBundleError`] for all the possible errors that may occur.
    ///
    #[cfg(feature = "clack-plugin")]
    pub fn load_from_clack<E: ::clack_plugin::entry::Entry>(
        path: &CStr,
    ) -> Result<Self, PluginBundleError> {
        let entry = E::new(path).map_err(|_| PluginBundleError::EntryInitFailed)?;
        let inner = PluginBundleInner::FromClack(clack_plugin::ClackEntry::new(entry));

        Ok(Self { inner })
    }

    /// Loads a CLAP bundle from a given symbol in a given [`libloading::Library`].
    ///
    /// This function takes ownership of the [`libloading::Library`] object, ensuring it stays
    /// properly loaded as long as the resulting [`PluginBundle`] or any plugin instance is kept
    /// alive.
    ///
    /// # Safety
    ///
    /// The given path must match the file location the library was loaded from. Moreover, the
    /// symbol named `symbol_name` must be a valid CLAP entry.
    ///
    /// Additionally, loading a non-compliant CLAP bundle may invalidate safety assumptions other
    /// APIs in this library rely on. See the [module docs](self)'s Safety section for more
    /// information.
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
    /// use std::ffi::CStr;
    /// use libloading::Library;
    /// use clack_host::prelude::PluginBundle;
    ///
    /// let path = "/home/user/.clap/u-he/libdiva.so";
    /// let lib = unsafe { Library::new(path) }.unwrap();
    /// let symbol_name = c"clap_entry";
    ///
    /// let bundle = unsafe { PluginBundle::load_from_symbol_in_library(path, lib, symbol_name)? };
    ///
    /// println!("Loaded bundle CLAP version: {}", bundle.version());
    /// # Ok(()) }
    /// ```
    #[cfg(feature = "libloading")]
    pub unsafe fn load_from_symbol_in_library<P: AsRef<std::ffi::OsStr>>(
        path: P,
        library: libloading::Library,
        symbol_name: &CStr,
    ) -> Result<Self, PluginBundleError> {
        use crate::bundle::library::PluginEntryLibrary;
        use std::ffi::CString;

        let path = path.as_ref();
        let path_cstr = CString::new(path.as_encoded_bytes())?;

        let library = PluginEntryLibrary::load_from_symbol_in_library(library, symbol_name)?;

        let inner = cache::load_from_library(library, &path_cstr)?;

        Ok(Self {
            inner: PluginBundleInner::Cached(inner),
        })
    }

    /// Loads a CLAP bundle from a `'static` [`EntryDescriptor`].
    ///
    /// Note that CLAP plugins loaded this way still need a valid path, as they may perform various
    /// filesystem operations relative to their bundle files.
    ///
    /// # Safety
    ///
    /// Loading a non-compliant CLAP bundle may invalidate safety assumptions other
    /// APIs in this library rely on. See the [module docs](self)'s Safety section for more
    /// information.
    ///
    /// # Errors
    ///
    /// This method returns an error if initializing the entry fails.
    /// See [`PluginBundleError`] for all the possible errors that may occur.
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
    /// let path = c"/home/user/.clap/u-he/libdiva.so";
    /// let bundle = unsafe { PluginBundle::load_from_raw(descriptor, path)? };
    ///
    /// println!("Loaded bundle CLAP version: {}", bundle.version());
    /// # Ok(()) }
    #[inline]
    pub unsafe fn load_from_raw(
        inner: &'static EntryDescriptor,
        plugin_path: &CStr,
    ) -> Result<Self, PluginBundleError> {
        Ok(Self {
            inner: PluginBundleInner::Cached(cache::load_from_raw(inner, plugin_path)?),
        })
    }

    /// Gets the raw, C-FFI plugin entry descriptor exposed by this bundle.
    #[inline]
    pub fn raw_entry(&self) -> &EntryDescriptor {
        match &self.inner {
            PluginBundleInner::Cached(entry) => entry.raw_entry(),
            #[cfg(feature = "clack-plugin")]
            PluginBundleInner::FromClack(_) => &clack_plugin::ClackEntry::DUMMY_DESCRIPTOR,
        }
    }

    /// Returns the [`Factory`] of type `F` exposed by this bundle, if it exists.
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
    /// use clack_host::factory::plugin::PluginFactory;
    /// use clack_host::prelude::PluginBundle;
    ///
    /// let bundle = unsafe { PluginBundle::load("/home/user/.clap/u-he/libdiva.so")? };
    /// let plugin_factory = bundle.get_factory::<PluginFactory>().unwrap();
    ///
    /// println!("Found {} plugins.", plugin_factory.plugin_count());
    /// # Ok(()) }
    /// ```
    pub fn get_factory<'a, F: Factory<'a>>(&'a self) -> Option<F> {
        let identifier = const { *F::IDENTIFIERS.first().unwrap() };
        match &self.inner {
            PluginBundleInner::Cached(entry) => {
                // SAFETY: this type ensures the function pointer is valid.
                let ptr = unsafe { entry.raw_entry().get_factory?(identifier.as_ptr()) };
                let ptr = NonNull::new(ptr.cast_mut())?;
                // SAFETY: Per the CLAP spec, if this pointer is non-null it has to be valid for reads
                let ptr = unsafe { RawFactoryPointer::from_raw(ptr.cast()) };

                // SAFETY: pointer was created using F's own identifier.
                Some(unsafe { F::from_raw(ptr) })
            }
            #[cfg(feature = "clack-plugin")]
            PluginBundleInner::FromClack(clack) => clack.get_factory(),
        }
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
    /// let bundle = unsafe { PluginBundle::load("/home/user/.clap/u-he/libdiva.so")? };
    /// let plugin_factory = bundle.get_plugin_factory().unwrap();
    ///
    /// println!("Found {} plugins.", plugin_factory.plugin_count());
    /// # Ok(()) }
    /// ```
    #[inline]
    pub fn get_plugin_factory(&self) -> Option<PluginFactory<'_>> {
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
#[non_exhaustive]
pub enum PluginBundleError {
    /// The dynamic library file could not be loaded.
    ///
    /// This contains the error type from the underlying
    /// [`libloading`](https://crates.io/crates/libloading) library.
    #[cfg(feature = "libloading")]
    LibraryLoadingError(libloading::Error),
    #[cfg(feature = "libloading")]
    /// The given path is not a valid C string.
    InvalidNulPath(std::ffi::NulError),
    /// The entry pointer exposed by the dynamic library file is `null`.
    NullEntryPointer,
    /// The exposed entry used an incompatible CLAP version.
    IncompatibleClapVersion {
        /// The CLAP version that the entry uses.
        ///
        /// See [`ClapVersion::CURRENT`] to get the current clap version.
        plugin_version: ClapVersion,
    },
    /// The entry's `init` method failed.
    EntryInitFailed,
}

impl Error for PluginBundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            #[cfg(feature = "libloading")]
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
            #[cfg(feature = "libloading")]
            PluginBundleError::InvalidNulPath(e) => {
                write!(f, "Invalid plugin descriptor path: {e}")
            }
            #[cfg(feature = "libloading")]
            PluginBundleError::LibraryLoadingError(e) => {
                write!(f, "Failed to load plugin descriptor library: {e}")
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

#[cfg(feature = "libloading")]
impl From<std::ffi::NulError> for PluginBundleError {
    #[inline]
    fn from(value: std::ffi::NulError) -> Self {
        Self::InvalidNulPath(value)
    }
}
