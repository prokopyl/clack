use crate::bundle::PluginBundleError;
use crate::bundle::entry_provider::EntryProvider;
use clack_common::entry::EntryDescriptor;
use libloading::Library;
use std::ffi::{CStr, OsStr};
use std::ptr::NonNull;

// TODO: bikeshade
pub struct LibraryEntry {
    _library: Library,
    entry_ptr: NonNull<EntryDescriptor>,
}

const SYMBOL_NAME: &CStr = c"clap_entry";
impl LibraryEntry {
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
