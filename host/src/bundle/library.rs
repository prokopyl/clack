use crate::bundle::entry::LoadedEntry;
use crate::bundle::PluginBundleError;
use clack_common::entry::EntryDescriptor;
use libloading::Library;
use selfie::refs::RefType;
use selfie::Selfie;
use stable_deref_trait::StableDeref;
use std::ffi::OsStr;
use std::ops::Deref;
use std::ptr::NonNull;

pub(crate) struct PluginEntryLibrary {
    _library: Library,
    entry_ptr: NonNull<EntryDescriptor>,
}

const SYMBOL_NAME: &[u8] = b"clap_entry\0";
impl PluginEntryLibrary {
    /// # Safety
    ///
    /// Loading an external library is inherently unsafe. Users must try their best to load only
    /// valid CLAP bundles.
    pub unsafe fn load(path: &OsStr) -> Result<Self, PluginBundleError> {
        let library = Library::new(path).map_err(PluginBundleError::LibraryLoadingError)?;

        let symbol = library
            .get::<*const EntryDescriptor>(SYMBOL_NAME)
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

// SAFETY: PluginEntryLibrary's referenced types are not affected by it being moved at all
unsafe impl StableDeref for PluginEntryLibrary {}

// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl Send for PluginEntryLibrary {}
// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl Sync for PluginEntryLibrary {}

pub(crate) struct LoadedEntryRef;

impl<'a> RefType<'a> for LoadedEntryRef {
    type Ref = LoadedEntry<'a>;
}

pub(crate) type LibraryEntry = Selfie<'static, PluginEntryLibrary, LoadedEntryRef>;
