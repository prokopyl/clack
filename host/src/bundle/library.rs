use crate::bundle::PluginBundleError;
use crate::bundle::entry_provider::EntryProvider;
use clack_common::entry::EntryDescriptor;
use libloading::Library;
use std::borrow::Cow;
use std::ffi::{CStr, CString, OsStr};
use std::path::Path;
use std::ptr::NonNull;

// TODO: bikeshade
pub struct LibraryEntry {
    _library: Library,
    entry_ptr: NonNull<EntryDescriptor>,
}

const SYMBOL_NAME: &CStr = c"clap_entry";
impl LibraryEntry {
    #[cfg(feature = "libloading")]
    pub(crate) fn resolve_path(path: &Path) -> Result<(Cow<'_, Path>, CString), PluginBundleError> {
        #[cfg(target_os = "macos")]
        {
            macos_resolve::resolve_path(path)
        }
        #[cfg(not(target_os = "macos"))]
        {
            // TODO: unwrap?
            let bundle_path = CString::new(path.to_string_lossy().into_owned())?;

            Ok((Cow::Borrowed(path), bundle_path))
        }
    }

    /// # Safety
    ///
    /// Loading an external library is inherently unsafe. Users must try their best to load only
    /// valid CLAP bundles.
    pub unsafe fn load_from_path(path: impl AsRef<OsStr>) -> Result<Self, PluginBundleError> {
        let library = Library::new(path).map_err(PluginBundleError::LibraryLoadingError)?;

        Self::load_from_library(library)
    }

    /// # Safety
    ///
    /// Loading an external library is inherently unsafe. Users must try their best to load only
    /// valid CLAP bundles.
    pub unsafe fn load_from_library(library: Library) -> Result<Self, PluginBundleError> {
        Self::load_from_symbol_in_library(library, SYMBOL_NAME)
    }

    /// # Safety
    ///
    /// Loading an external library is inherently unsafe. Users must try their best to load only
    /// valid CLAP bundles.
    pub unsafe fn load_from_symbol_in_library(
        library: Library,
        symbol_name: &CStr,
    ) -> Result<Self, PluginBundleError> {
        let symbol = library
            .get::<*const EntryDescriptor>(symbol_name.to_bytes_with_nul())
            .map_err(PluginBundleError::LibraryLoadingError)?;

        let entry_ptr = NonNull::new(*symbol as *mut EntryDescriptor)
            .ok_or(PluginBundleError::NullEntryPointer)?;

        Ok(Self {
            _library: library,
            entry_ptr,
        })
    }
}

// SAFETY: TODO
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

    pub(super) fn resolve_path(path: &Path) -> Result<(Cow<'_, Path>, CString), PluginBundleError> {
        if std::fs::metadata(path)?.is_dir() {
            // The given path is to the bundle.
            let executable_path =
                executable_path_from_bundle(path).ok_or(PluginBundleError::ResolveFailed)?;

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
        if let Some(bundle_path) = bundle_path_from_clap_package_ancestor(path) {
            return bundle_path;
        };

        if let Some(bundle_path) = bundle_path_from_structure(path) {
            return bundle_path;
        }

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
