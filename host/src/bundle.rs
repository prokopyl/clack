use clap_sys::entry::clap_plugin_entry;
use libloading::Library;
use std::error::Error;
use std::ffi::{CString, NulError, OsStr};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;

use clack_common::factory::Factory;
use selfie::refs::RefType;
use selfie::Selfie;
use stable_deref_trait::StableDeref;
use std::ptr::NonNull;

pub use clack_common::bundle::*;

pub(crate) struct PluginEntryLibrary {
    _library: Library,
    entry_ptr: NonNull<clap_plugin_entry>,
}

const SYMBOL_NAME: &[u8] = b"clap_entry\0";
impl PluginEntryLibrary {
    pub fn load(path: &OsStr) -> Result<Self, PluginEntryError> {
        let library =
            unsafe { Library::new(path) }.map_err(PluginEntryError::LibraryLoadingError)?;

        let symbol = unsafe { library.get::<*const clap_plugin_entry>(SYMBOL_NAME) }
            .map_err(PluginEntryError::LibraryLoadingError)?;

        let entry_ptr = NonNull::new(*symbol as *mut clap_plugin_entry)
            .ok_or(PluginEntryError::NullEntryPointer)?;

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

pub(crate) struct LoadedEntry<'a> {
    entry: &'a clap_plugin_entry,
}

impl<'a> LoadedEntry<'a> {
    pub unsafe fn load(entry: &'a clap_plugin_entry, path: &str) -> Result<Self, PluginEntryError> {
        let path = CString::new(path).map_err(PluginEntryError::NulDescriptorPath)?;

        if !(entry.init)(path.as_ptr()) {
            return Err(PluginEntryError::EntryInitFailed);
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

pub(crate) struct LoadedEntryRef;

impl<'a> RefType<'a> for LoadedEntryRef {
    type Ref = LoadedEntry<'a>;
}

enum EntrySource {
    FromRaw(LoadedEntry<'static>),
    FromLibrary(Selfie<'static, PluginEntryLibrary, LoadedEntryRef>),
}

#[derive(Clone)]
pub struct PluginBundle {
    inner: Pin<Arc<EntrySource>>,
}

impl PluginBundle {
    pub fn load<P: AsRef<OsStr>>(path: P) -> Result<Self, PluginEntryError> {
        let path = path.as_ref();
        let path_str = path.to_str().ok_or(PluginEntryError::InvalidUtf8Path)?;

        let library = Pin::new(PluginEntryLibrary::load(path)?);

        let inner = Arc::pin(EntrySource::FromLibrary(Selfie::try_new(
            library,
            |entry| unsafe { LoadedEntry::load(entry, path_str) },
        )?));

        Ok(Self { inner })
    }

    /// # Safety
    /// Must only be called once for a given descriptor, else entry could be init'd multiple times
    #[inline]
    pub unsafe fn load_from_raw(
        inner: &'static PluginEntryDescriptor,
        plugin_path: &str,
    ) -> Result<Self, PluginEntryError> {
        Ok(Self {
            inner: Arc::pin(EntrySource::FromRaw(LoadedEntry::load(inner, plugin_path)?)),
        })
    }

    #[inline]
    pub fn raw_entry(&self) -> &clap_plugin_entry {
        match &self.inner.as_ref().get_ref() {
            EntrySource::FromRaw(raw) => raw.entry(),
            EntrySource::FromLibrary(bundle) => bundle.with_referential(|e| e.entry()),
        }
    }

    pub fn get_factory<F: Factory>(&self) -> Option<&F> {
        let ptr = unsafe { (self.raw_entry().get_factory)(F::IDENTIFIER as *const _) } as *mut _;
        NonNull::new(ptr).map(|p| unsafe { F::from_factory_ptr(p) })
    }

    #[inline]
    pub fn version(&self) -> ClapVersion {
        self.raw_entry().clap_version
    }
}

#[derive(Debug)]
pub enum PluginEntryError {
    EntryInitFailed,
    NulDescriptorPath(NulError),
    NullEntryPointer,
    InvalidUtf8Path,
    LibraryLoadingError(libloading::Error),
}

impl Error for PluginEntryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PluginEntryError::NulDescriptorPath(e) => Some(e),
            PluginEntryError::LibraryLoadingError(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for PluginEntryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginEntryError::EntryInitFailed => f.write_str("Plugin entry initialization failed"),
            PluginEntryError::NulDescriptorPath(e) => {
                write!(f, "Invalid plugin descriptor path: {}", e)
            }
            PluginEntryError::LibraryLoadingError(e) => {
                write!(f, "Failed to load plugin descriptor library: {}", e)
            }
            PluginEntryError::InvalidUtf8Path => {
                f.write_str("Plugin descriptor path contains invalid UTF-8")
            }
            PluginEntryError::NullEntryPointer => f.write_str("Plugin entry pointer is null"),
        }
    }
}
