use crate::bundle::PluginBundleError;
use clack_common::entry::EntryDescriptor;
use libloading::Library;
use std::ffi::{CStr, OsStr};
use std::ops::Deref;
use std::ptr::NonNull;

pub(crate) struct PluginEntryLibrary {
    _library: Library,
    entry_ptr: NonNull<EntryDescriptor>,
}

// SAFETY: this has a null byte at the end
const SYMBOL_NAME: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"clap_entry\0") };
impl PluginEntryLibrary {
    /// # Safety
    ///
    /// Loading an external library is inherently unsafe. Users must try their best to load only
    /// valid CLAP bundles.
    pub unsafe fn load(path: &OsStr) -> Result<Self, PluginBundleError> {
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

    #[inline]
    pub const fn entry(&self) -> &EntryDescriptor {
        // SAFETY: this type's only constructor guarantees this pointer is valid
        unsafe { self.entry_ptr.as_ref() }
    }
}

impl Deref for PluginEntryLibrary {
    type Target = EntryDescriptor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.entry()
    }
}

// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl Send for PluginEntryLibrary {}
// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl Sync for PluginEntryLibrary {}
