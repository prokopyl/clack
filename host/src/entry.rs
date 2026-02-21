#![warn(missing_docs)]

//! Loading and handling of CLAP plugin entry files.
//!
//! On Windows and Linux (and other non-macOS UNIXes), CLAP plugins are distributed as prebuilt
//! dynamically-loaded libraries (usually `.dll` or `.so` files) with a `.clap` extension.
//! They expose a single [`EntryDescriptor`], which, once initialized, acts as the entry
//! point for the host to read into the library file.
//!
//! On macOS, CLAP plugins are distributed as a standard
//! [bundle](https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/AboutBundles/AboutBundles.html),
//! which contains the aforementioned dynamically-loaded libraries as its executable file.
//!
//! CLAP plugin entries expose implementations of various standard [factories](Factory), which are
//! singletons implementing various functionalities. The most relevant is the [`PluginFactory`],
//! which allows to list and instantiate plugins. See the [`factory`](crate::factory) module
//! documentation to learn more about factories.
//!
//! Clack handles all of this functionality through the [`PluginEntry`] type, which exposes all
//! the entry's factory implementation, and allow the entry to be loaded in different ways:
//!
//! * From a file, using [`PluginEntry::load`].
//!
//!   This is the most common usage, as it allows to load
//!   third-party CLAP entries present anywhere on the file system, which is most likely the
//!   functionality "CLAP plugin support" implies for most hosts.
//!
//! * From an [`EntryProvider`] type, using [`PluginEntry::load_from`].
//!
//!   This is a more advanced usage, and it allows host implementations to not use Clack's
//!   implementation of entry loading
//!   (which uses [`libloading`](https://crates.io/crates/libloading) under the hood), and implement
//!   their own instead.
//!
//!   It also allows host implementations to customize Clack's implementation of entry loading by
//!   leveraging the [`LibraryEntry`] built-in entry provider type.
//!
//! * From a `clack-plugin`'s `Entry` type, using [`PluginEntry::load_from_clack`].
//!
//!   This allows to load plugins that have been statically built into the host's binary
//!   (i.e. built-in plugins) without having to distribute them in separate
//!   files, or to perform any filesystem access or plugin discovery.
//!
//!   This is also the only way to load a CLAP plugin using only safe code, since it leverages
//!   `clap-plugin`'s safety guarantees to enforce that plugins loaded this way cannot cause
//!   Undefined Behavior.
//!
//! See the [`PluginEntry`]'s type documentation for examples.
//!
//! # Safety
//!
//! All functions that produce [`PluginEntry`]s from a CLAP entry file or pointer are inherently
//! unsafe.
//!
//! Most APIs in this crate operate under the assumption that entries and plugins are compliant
//! to the CLAP specification, and using a number of those APIs can easily result in
//! Undefined Behavior if operating on non-compliant plugins.
//!
//! Therefore, the safe APIs in this crate are safeguarding the host implementation, not the
//! plugins it loads. As soon as a plugin is loaded, Undefined Behavior is possible to be triggered,
//! regardless of the host's implementation.
//!
//! Users needing to safeguard against crashes or other kinds of UB from plugins from affecting the
//! rest of their application should consider using additional process isolation techniques.
//!
//! # Plugin entry discovery
//!
//! Clack itself does not implement any utilities to aid host implementations with discovering
//! which CLAP entry files are available to be loaded on the filesystem.
//!
//! However, you may use the separate [`clack-finder`](https://github.com/prokopyl/clack/tree/main/finder)
//! library to perform this task.
//!
//! Refer to the
//! [CLAP specification](https://github.com/free-audio/clap/blob/main/include/clap/entry.h) for more
//! information about standard search paths and the general discovery process.

use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::ptr::NonNull;

mod cache;
mod entry_provider;
mod loaded_entry;

#[cfg(feature = "libloading")]
mod library;
#[cfg(feature = "libloading")]
pub use library::LibraryEntry;

#[cfg(feature = "clack-plugin")]
mod clack_plugin;

#[cfg(test)]
#[allow(missing_docs)]
pub mod diva_stub;

use crate::entry::cache::CachedEntry;
use crate::factory::{Factory, plugin::PluginFactory};
pub use clack_common::entry::*;
use clack_common::factory::RawFactoryPointer;
use clack_common::utils::ClapVersion;
pub use entry_provider::EntryProvider;

/// A handle to a loaded CLAP plugin entry.
///
/// This allows getting all the [factories](Factory) exposed by the entry, mainly the
/// [`PluginFactory`] which allows to list plugin instances.
///
/// This is only a lightweight handle: plugin entries are only loaded once, and the [`Clone`]
/// operation on this type only clones the handle.
///
/// Plugin entries are only unloaded when all handles are dropped and all associated instances
/// are unloaded.
///
/// A [`PluginEntry`] can also be loaded from a static [`EntryDescriptor`] instead of a file.
/// See [`PluginEntry::load_from_raw`].
///
/// See the [module docs](crate::entry) for more information about CLAP entries.
///
/// # Example
///
/// ```no_run
/// # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use clack_host::prelude::PluginEntry;
///
/// let entry = unsafe { PluginEntry::load("/home/user/.clap/u-he/libdiva.so")? };
/// let plugin_factory = entry.get_plugin_factory().unwrap();
///
/// println!("Loaded entry CLAP version: {}", entry.version());
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct PluginEntry {
    inner: PluginEntryInner,
}

#[derive(Clone)]
enum PluginEntryInner {
    Cached(CachedEntry),
    #[cfg(feature = "clack-plugin")]
    FromClack(clack_plugin::ClackEntry),
}

impl PluginEntry {
    /// Loads and initializes a CLAP entry from a dynamic library file located at the given path.
    ///
    /// This function also initializes the loaded entry, using the provided path, and returns a
    /// handle to it.
    ///
    /// If you wish to initialize the entry with a different path than the one of the dynamic
    /// library file it comes from, you can use [`load_from`](Self::load_from) with a
    /// [`LibraryEntry`] as an argument.
    ///
    /// # Platform compatibility notes
    ///
    /// On macOS, plugins are not packaged as their dynamic library file directly, but as a standard
    /// bundle which contains the dynamic library file instead.
    ///
    /// For convenience, on macOS this function can accept both a path to the plugin bundle, or a
    /// path to the dynamic library file itself.
    ///
    /// If a path to a bundle is given to this function, it will use its `Info.plist` file to locate
    /// the bundle's executable and load that. It will then use the given path to the bundle to
    /// initialize the entry.
    ///
    /// If a path to a file is given to this function, it will load that as the dynamic library file,
    /// and attempt to find or reconstruct the bundle's path from the executable path. It will then
    /// use that path to initialize the entry.
    ///
    /// If you have access to both paths and want to eliminate this guesswork, you should use
    /// [`load_from`](Self::load_from) with a [`LibraryEntry`] as an argument instead.
    ///
    /// # Safety
    ///
    /// This function loads an external library object file, which is inherently unsafe, as even
    /// just loading it can trigger any behavior in your application, including Undefined Behavior.
    ///
    /// Additionally, loading a non-compliant CLAP entry may invalidate safety assumptions other
    /// APIs in this library rely on. See the [module docs](self)'s Safety section for more
    /// information.
    ///
    /// # Errors
    ///
    /// This method returns an error if loading the entry fails.
    /// See [`PluginEntryError`] for all the possible errors that may occur.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use clack_host::prelude::PluginEntry;
    ///
    /// let entry = unsafe { PluginEntry::load("/home/user/.clap/u-he/libdiva.so")? };
    ///
    /// println!("Loaded entry CLAP version: {}", entry.version());
    /// # Ok(()) }
    /// ```
    #[cfg(feature = "libloading")]
    pub unsafe fn load<P: AsRef<std::ffi::OsStr>>(path: P) -> Result<Self, PluginEntryError> {
        use crate::entry::library::LibraryEntry;
        use std::path::Path;

        let path = Path::new(&path);

        let (library_path, bundle_path) = LibraryEntry::resolve_path(path)?;

        let library = LibraryEntry::load_from_path(library_path.as_ref())?;
        Self::load_from(library, &bundle_path)
    }

    /// Initializes a CLAP entry exposed by the given [`EntryProvider`].
    ///
    /// This takes ownership of the given `provider`, and will keep it alive as long as there are
    /// [`PluginEntry`] references to it, or as long as [`PluginInstance`s](crate::plugin::PluginInstance)
    /// (or other similar objects) created by this entry also exist.
    ///
    /// # Safety
    ///
    /// Loading a non-compliant CLAP entry may invalidate safety assumptions other
    /// APIs in this library rely on. See the [module docs](self)'s Safety section for more
    /// information.
    ///
    /// # Errors
    ///
    /// This method returns an error if initializing the entry fails.
    /// See [`PluginEntryError`] for all the possible errors that may occur.
    pub unsafe fn load_from(
        provider: impl EntryProvider,
        path: &CStr,
    ) -> Result<Self, PluginEntryError> {
        let inner = cache::get_or_init(provider, path)?;

        Ok(Self {
            inner: PluginEntryInner::Cached(inner),
        })
    }

    /// Loads a CLAP entry from an [`Entry`](::clack_plugin::entry::Entry) created by [`clack_plugin`](::clack_plugin).
    ///
    /// Note that CLAP plugins loaded this way still need a valid path to initialize, as they may
    /// perform various filesystem operations relative to their entry files.
    ///
    /// Note that unlike other methods to load [`PluginEntry`]s, this method is completely safe, as
    /// it can rely on the safety guarantees provided by [`clack_plugin`](::clack_plugin).
    ///
    /// # Errors
    ///
    /// This method returns an error if initializing the entry fails.
    /// See [`PluginEntryError`] for all the possible errors that may occur.
    ///
    #[cfg(feature = "clack-plugin")]
    pub fn load_from_clack<E: ::clack_plugin::entry::Entry>(
        path: &CStr,
    ) -> Result<Self, PluginEntryError> {
        let entry = E::new(path).map_err(|_| PluginEntryError::EntryInitFailed)?;
        let inner = PluginEntryInner::FromClack(clack_plugin::ClackEntry::new(entry));

        Ok(Self { inner })
    }

    /// Loads a CLAP entry from a `'static` [`EntryDescriptor`].
    ///
    /// Note that CLAP plugins loaded this way still need a valid path, as they may perform various
    /// filesystem operations relative to their entry files.
    ///
    /// # Safety
    ///
    /// Loading a non-compliant CLAP entry may invalidate safety assumptions other
    /// APIs in this library rely on. See the [module docs](self)'s Safety section for more
    /// information.
    ///
    /// # Errors
    ///
    /// This method returns an error if initializing the entry fails.
    /// See [`PluginEntryError`] for all the possible errors that may occur.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clack_host::entry::EntryDescriptor;
    /// use clack_host::prelude::PluginEntry;
    /// # pub fn foo(descriptor: &'static EntryDescriptor) -> Result<(), Box<dyn std::error::Error>> {
    ///
    /// let descriptor: &'static EntryDescriptor = /* ... */
    /// # descriptor;
    ///
    /// let path = c"/home/user/.clap/u-he/libdiva.so";
    /// let entry = unsafe { PluginEntry::load_from_raw(descriptor, path)? };
    ///
    /// println!("Loaded entry CLAP version: {}", entry.version());
    /// # Ok(()) }
    #[inline]
    pub unsafe fn load_from_raw(
        inner: &'static EntryDescriptor,
        plugin_path: &CStr,
    ) -> Result<Self, PluginEntryError> {
        Self::load_from(inner, plugin_path)
    }

    /// Gets the raw, C-FFI plugin entry descriptor exposed by this entry.
    #[inline]
    pub fn raw_entry(&self) -> &EntryDescriptor {
        match &self.inner {
            PluginEntryInner::Cached(entry) => entry.as_ref(),
            #[cfg(feature = "clack-plugin")]
            PluginEntryInner::FromClack(_) => &clack_plugin::ClackEntry::DUMMY_DESCRIPTOR,
        }
    }

    /// Returns the [`Factory`] of type `F` exposed by this entry, if it exists.
    ///
    /// If this entry does not expose a factory of the requested type, [`None`] is returned.
    ///
    /// If you are looking to fetch the entry's [`PluginFactory`], you can also use the
    /// [`get_plugin_factory`](PluginEntry::get_plugin_factory) method, which is just a convenience
    /// wrapper around this method.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use clack_host::factory::plugin::PluginFactory;
    /// use clack_host::prelude::PluginEntry;
    ///
    /// let entry = unsafe { PluginEntry::load("/home/user/.clap/u-he/libdiva.so")? };
    /// let plugin_factory = entry.get_factory::<PluginFactory>().unwrap();
    ///
    /// println!("Found {} plugins.", plugin_factory.plugin_count());
    /// # Ok(()) }
    /// ```
    pub fn get_factory<'a, F: Factory<'a>>(&'a self) -> Option<F> {
        let identifier = const { *F::IDENTIFIERS.first().unwrap() };
        match &self.inner {
            PluginEntryInner::Cached(entry) => {
                // SAFETY: this type ensures the function pointer is valid.
                let ptr = unsafe { entry.get().get_factory?(identifier.as_ptr()) };
                let ptr = NonNull::new(ptr.cast_mut())?;
                // SAFETY: Per the CLAP spec, if this pointer is non-null it has to be valid for reads
                let ptr = unsafe { RawFactoryPointer::from_raw(ptr.cast()) };

                // SAFETY: pointer was created using F's own identifier.
                Some(unsafe { F::from_raw(ptr) })
            }
            #[cfg(feature = "clack-plugin")]
            PluginEntryInner::FromClack(clack) => clack.get_factory(),
        }
    }

    /// Returns the [`PluginFactory`] exposed by this entry, if it exists.
    ///
    /// If this entry does not expose a [`PluginFactory`], [`None`] is returned.
    ///
    /// This is a convenience method, and is equivalent to calling
    /// [`get_factory`](PluginEntry::get_factory) with a [`PluginFactory`] type parameter.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use clack_host::prelude::PluginEntry;
    ///
    /// let entry = unsafe { PluginEntry::load("/home/user/.clap/u-he/libdiva.so")? };
    /// let plugin_factory = entry.get_plugin_factory().unwrap();
    ///
    /// println!("Found {} plugins.", plugin_factory.plugin_count());
    /// # Ok(()) }
    /// ```
    #[inline]
    pub fn get_plugin_factory(&self) -> Option<PluginFactory<'_>> {
        self.get_factory()
    }

    /// Returns the CLAP version used by this entry.
    #[inline]
    pub fn version(&self) -> ClapVersion {
        ClapVersion::from_raw(self.raw_entry().clap_version)
    }
}

/// Errors that can occur while loading a [`PluginEntry`].
///
/// See [`PluginEntry::load`] and [`PluginEntry::load_from_raw`].
#[derive(Debug)]
#[non_exhaustive]
pub enum PluginEntryError {
    /// The dynamic library file could not be loaded.
    ///
    /// This contains the error type from the underlying
    /// [`libloading`](https://crates.io/crates/libloading) library.
    #[cfg(feature = "libloading")]
    LibraryLoadingError(libloading::Error),
    #[cfg(feature = "libloading")]
    /// The given path is not a valid C string.
    InvalidNulPath(std::ffi::NulError),
    /// An I/O error occurred.
    Io(std::io::Error),
    /// Failed to resolve a CLAP entry path.
    ///
    /// This error can only occur on macOS.
    ResolveFailed,
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

impl Error for PluginEntryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            #[cfg(feature = "libloading")]
            PluginEntryError::InvalidNulPath(e) => Some(e),
            #[cfg(feature = "libloading")]
            PluginEntryError::LibraryLoadingError(e) => Some(e),
            PluginEntryError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for PluginEntryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginEntryError::EntryInitFailed => f.write_str("Plugin entry initialization failed"),
            #[cfg(feature = "libloading")]
            PluginEntryError::InvalidNulPath(e) => {
                write!(f, "Invalid plugin descriptor path: {e}")
            }
            #[cfg(feature = "libloading")]
            PluginEntryError::LibraryLoadingError(e) => {
                write!(f, "Failed to load plugin descriptor library: {e}")
            }
            PluginEntryError::Io(e) => Display::fmt(e, f),
            PluginEntryError::NullEntryPointer => f.write_str("Plugin entry pointer is null"),
            PluginEntryError::ResolveFailed => f.write_str("Failed to resolve plugin entry path"),
            PluginEntryError::IncompatibleClapVersion { plugin_version } => write!(
                f,
                "Incompatible CLAP version: plugin is v{}, host is v{}",
                plugin_version,
                ClapVersion::CURRENT
            ),
        }
    }
}

#[cfg(feature = "libloading")]
impl From<std::ffi::NulError> for PluginEntryError {
    #[inline]
    fn from(value: std::ffi::NulError) -> Self {
        Self::InvalidNulPath(value)
    }
}

impl From<std::io::Error> for PluginEntryError {
    #[inline]
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
