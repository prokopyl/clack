use crate::entry::PluginEntryError;
use crate::entry::entry_provider::EntryProvider;
use clack_common::entry::EntryDescriptor;
use libloading::Library;
use std::borrow::Cow;
use std::ffi::{CStr, CString, OsStr};
use std::path::Path;
use std::ptr::NonNull;

/// An [`EntryProvider`] which loads an entry from a dynamic library file.
///
/// This uses the [`libloading`] library under the hood. It will not be available if the associated
/// `libloading` Cargo feature of `clack-host` is not enabled.
pub struct LibraryEntry {
    _library: Library,
    entry_ptr: NonNull<EntryDescriptor>,
}

const SYMBOL_NAME: &CStr = c"clap_entry";
impl LibraryEntry {
    #[cfg(feature = "libloading")]
    pub(crate) fn resolve_path(path: &Path) -> Result<(Cow<'_, Path>, CString), PluginEntryError> {
        #[cfg(target_os = "macos")]
        {
            macos_resolve::resolve_path(path)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let bundle_path = CString::new(path.to_string_lossy().into_owned())?;

            Ok((Cow::Borrowed(path), bundle_path))
        }
    }

    /// Loads a CLAP entry from a dynamic library file located at the given path.
    ///
    /// # Errors
    ///
    /// This function will return an error if library file could not be loaded, if the `clap_entry`
    /// symbol could not be extracted from the given library, or if the symbol's value is NULL.
    ///
    /// # Safety
    ///
    /// This function loads an external library object file, which is inherently unsafe, as even
    /// just loading it can trigger any behavior in your application, including Undefined Behavior.
    ///
    /// Additionally, users
    pub unsafe fn load_from_path(path: impl AsRef<OsStr>) -> Result<Self, PluginEntryError> {
        let library = Library::new(path).map_err(PluginEntryError::LibraryLoadingError)?;

        Self::load_from_library(library)
    }

    /// Wraps a given [`Library`] object to load a CLAP entry from.
    ///
    /// This function will look for the standard `clap_entry` symbol to get the [`EntryDescriptor`]
    /// pointer from. If you need to load that pointer from another, non-standard symbol, see
    /// [`load_from_symbol_in_library`](Self::load_from_symbol_in_library).
    ///
    /// # Errors
    ///
    /// This function will return an error if the `clap_entry` symbol could not be extracted from the given
    /// library, or if the symbol's value is NULL.
    ///
    /// # Safety
    ///
    /// Users of this function must ensure the symbol `clap_entry` in the given library is ABI
    /// compatible with a pointer to an [`EntryDescriptor`].
    pub unsafe fn load_from_library(library: Library) -> Result<Self, PluginEntryError> {
        Self::load_from_symbol_in_library(library, SYMBOL_NAME)
    }

    /// Wraps a given [`Library`] object to load a CLAP entry from a given symbol it contains.
    ///
    /// # Errors
    ///
    /// This function will return an error if the symbol could not be extracted from the given
    /// library, or if the symbol's value is NULL.
    ///
    /// # Safety
    ///
    /// Users of this function must ensure the given symbol `symbol_name` in the given library is ABI
    /// compatible with a pointer to an [`EntryDescriptor`].
    pub unsafe fn load_from_symbol_in_library(
        library: Library,
        symbol_name: &CStr,
    ) -> Result<Self, PluginEntryError> {
        let symbol = library
            .get::<*mut EntryDescriptor>(symbol_name.to_bytes_with_nul())
            .map_err(PluginEntryError::LibraryLoadingError)?;

        let entry_ptr = NonNull::new(*symbol).ok_or(PluginEntryError::NullEntryPointer)?;

        Ok(Self {
            _library: library,
            entry_ptr,
        })
    }
}

// SAFETY: The pointer's value never changes, and since we hold onto the Library it comes from, it
// will not be unloaded as long as this is alive.
unsafe impl EntryProvider for LibraryEntry {
    #[inline]
    fn entry_pointer(&self) -> NonNull<EntryDescriptor> {
        self.entry_ptr
    }
}

// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl Send for LibraryEntry {}
// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl Sync for LibraryEntry {}

#[cfg(target_os = "macos")]
mod macos_resolve {
    use super::*;
    use std::path::{Path, PathBuf};

    pub(super) fn resolve_path(path: &Path) -> Result<(Cow<'_, Path>, CString), PluginEntryError> {
        if std::fs::metadata(path)?.is_dir() {
            // The given path is to the bundle.
            let executable_path =
                executable_path_from_bundle(path).ok_or(PluginEntryError::ResolveFailed)?;

            let bundle_path = CString::new(path.to_string_lossy().into_owned())?;

            Ok((Cow::Owned(executable_path), bundle_path))
        } else {
            // The given path is to the executable.
            let bundle_path = bundle_path_from_executable(path);
            let bundle_path = CString::new(bundle_path.to_string_lossy().into_owned())?;

            Ok((Cow::Borrowed(path), bundle_path))
        }
    }

    fn executable_path_from_bundle(path: &Path) -> Option<PathBuf> {
        use objc2_foundation::{NSBundle, NSURL};

        let url = NSURL::from_directory_path(path)?;
        let bundle = NSBundle::bundleWithURL(&url)?;

        let executable_url = bundle.executableURL()?.absoluteURL()?;
        let executable_path = executable_url.to_file_path()?;

        Some(executable_path)
    }

    fn bundle_path_from_executable(path: &Path) -> &Path {
        if let Some(bundle_path) = bundle_path_from_structure(path) {
            return bundle_path;
        };

        if let Some(bundle_path) = bundle_path_from_clap_package_ancestor(path) {
            return bundle_path;
        };

        path
    }

    fn bundle_path_from_structure(path: &Path) -> Option<&Path> {
        let mut ancestors = path.ancestors();
        if ancestors.next()? != "MacOS" {
            return None;
        }

        if ancestors.next()? != "Contents" {
            return None;
        }

        ancestors.next()
    }

    fn bundle_path_from_clap_package_ancestor(path: &Path) -> Option<&Path> {
        path.ancestors()
            .find(|a| a.extension().is_some_and(|e| e == "clap"))
    }
}
