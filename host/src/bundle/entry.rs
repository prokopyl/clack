use crate::bundle::PluginBundleError;
use clack_common::version::ClapVersion;
use clap_sys::entry::clap_plugin_entry;
use libloading::Library;
use selfie::refs::RefType;
use selfie::Selfie;
use stable_deref_trait::StableDeref;
use std::ffi::{CString, OsStr};
use std::ops::Deref;
use std::ptr::NonNull;

pub struct PluginEntryLibrary {
    _library: Library,
    entry_ptr: NonNull<clap_plugin_entry>,
}

const SYMBOL_NAME: &[u8] = b"clap_entry\0";
impl PluginEntryLibrary {
    pub fn load(path: &OsStr) -> Result<Self, PluginBundleError> {
        let library =
            unsafe { Library::new(path) }.map_err(PluginBundleError::LibraryLoadingError)?;

        let symbol = unsafe { library.get::<*const clap_plugin_entry>(SYMBOL_NAME) }
            .map_err(PluginBundleError::LibraryLoadingError)?;

        let entry_ptr = NonNull::new(*symbol as *mut clap_plugin_entry)
            .ok_or(PluginBundleError::NullEntryPointer)?;

        Ok(Self {
            _library: library,
            entry_ptr,
        })
    }

    #[inline]
    pub fn entry(&self) -> &clap_plugin_entry {
        unsafe { self.entry_ptr.as_ref() }
    }
}

impl Deref for PluginEntryLibrary {
    type Target = clap_plugin_entry;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.entry()
    }
}

unsafe impl StableDeref for PluginEntryLibrary {}

pub struct LoadedEntry<'a> {
    entry: &'a clap_plugin_entry,
}

impl<'a> LoadedEntry<'a> {
    pub unsafe fn load(
        entry: &'a clap_plugin_entry,
        path: &str,
    ) -> Result<Self, PluginBundleError> {
        let plugin_version = ClapVersion::from_raw(entry.clap_version);
        if !plugin_version.is_compatible() {
            return Err(PluginBundleError::IncompatibleClapVersion { plugin_version });
        }

        let path = CString::new(path).map_err(PluginBundleError::NulDescriptorPath)?;

        if !(entry.init)(path.as_ptr()) {
            return Err(PluginBundleError::EntryInitFailed);
        }

        Ok(Self { entry })
    }

    #[inline]
    pub fn entry(&self) -> &'a clap_plugin_entry {
        self.entry
    }
}

impl<'a> Drop for LoadedEntry<'a> {
    fn drop(&mut self) {
        unsafe { (self.entry.deinit)() }
    }
}

// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl<'a> Send for LoadedEntry<'a> {}
unsafe impl<'a> Sync for LoadedEntry<'a> {}

pub struct LoadedEntryRef;

impl<'a> RefType<'a> for LoadedEntryRef {
    type Ref = LoadedEntry<'a>;
}

pub enum EntrySource {
    FromRaw(LoadedEntry<'static>),
    FromLibrary(Selfie<'static, PluginEntryLibrary, LoadedEntryRef>),
}
