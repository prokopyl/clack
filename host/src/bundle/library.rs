use crate::bundle::entry::LoadedEntry;
use crate::bundle::PluginBundleError;
use clack_common::entry::EntryDescriptor;
use libloading::Library;
use selfie::refs::RefType;
use selfie::Selfie;
use stable_deref_trait::StableDeref;
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
    pub fn load(path: &OsStr) -> Result<Self, PluginBundleError> {
        let library =
            unsafe { Library::new(path) }.map_err(PluginBundleError::LibraryLoadingError)?;

        Self::load_from_library(library)
    }

    pub fn load_from_library(library: Library) -> Result<Self, PluginBundleError> {
        unsafe { Self::load_from_symbol_in_library(library, SYMBOL_NAME) }
    }

    pub unsafe fn load_from_symbol_in_library(
        library: Library,
        symbol_name: &CStr,
    ) -> Result<Self, PluginBundleError> {
        let symbol =
            unsafe { library.get::<*const EntryDescriptor>(symbol_name.to_bytes_with_nul()) }
                .map_err(PluginBundleError::LibraryLoadingError)?;

        let entry_ptr = NonNull::new(*symbol as *mut EntryDescriptor)
            .ok_or(PluginBundleError::NullEntryPointer)?;

        Ok(Self {
            _library: library,
            entry_ptr,
        })
    }

    #[inline]
    pub fn entry(&self) -> &EntryDescriptor {
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

unsafe impl StableDeref for PluginEntryLibrary {}

// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl Send for PluginEntryLibrary {}
unsafe impl Sync for PluginEntryLibrary {}

pub(crate) struct LoadedEntryRef;

impl<'a> RefType<'a> for LoadedEntryRef {
    type Ref = LoadedEntry<'a>;
}

pub(crate) type LibraryEntry = Selfie<'static, PluginEntryLibrary, LoadedEntryRef>;
