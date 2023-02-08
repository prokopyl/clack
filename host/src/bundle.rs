use std::error::Error;
use std::ffi::{NulError, OsStr};
use std::fmt::{Display, Formatter};
use std::pin::Pin;
use std::sync::Arc;

use clack_common::factory::Factory;
use entry::*;
use selfie::Selfie;
use std::ptr::NonNull;

mod entry;

#[cfg(test)]
mod diva_stub;

use crate::factory::PluginFactory;
pub use clack_common::bundle::*;
use clack_common::utils::ClapVersion;

#[derive(Clone)]
pub struct PluginBundle {
    inner: Pin<Arc<EntrySource>>,
}

impl PluginBundle {
    pub fn load<P: AsRef<OsStr>>(path: P) -> Result<Self, PluginBundleError> {
        let path = path.as_ref();
        let path_str = path.to_str().ok_or(PluginBundleError::InvalidUtf8Path)?;

        let library = Pin::new(PluginEntryLibrary::load(path)?);

        let inner = Arc::pin(EntrySource::FromLibrary(
            Selfie::try_new(library, |entry| unsafe {
                LoadedEntry::load(entry, path_str)
            })
            // The library can be discarded completely
            .map_err(|e| e.error)?,
        ));

        Ok(Self { inner })
    }

    /// # Safety
    /// Must only be called once for a given descriptor, else entry could be init'd multiple times
    #[inline]
    pub unsafe fn load_from_raw(
        inner: &'static PluginEntryDescriptor,
        plugin_path: &str,
    ) -> Result<Self, PluginBundleError> {
        Ok(Self {
            inner: Arc::pin(EntrySource::FromRaw(LoadedEntry::load(inner, plugin_path)?)),
        })
    }

    #[inline]
    pub fn raw_entry(&self) -> &PluginEntryDescriptor {
        match &self.inner.as_ref().get_ref() {
            EntrySource::FromRaw(raw) => raw.entry(),
            EntrySource::FromLibrary(bundle) => bundle.with_referential(|e| e.entry()),
        }
    }

    pub fn get_factory<F: Factory>(&self) -> Option<&F> {
        let ptr = unsafe { (self.raw_entry().get_factory?)(F::IDENTIFIER.as_ptr()) } as *mut _;
        NonNull::new(ptr).map(|p| unsafe { F::from_factory_ptr(p) })
    }

    #[inline]
    pub fn get_plugin_factory(&self) -> Option<&PluginFactory> {
        self.get_factory()
    }

    #[inline]
    pub fn version(&self) -> ClapVersion {
        ClapVersion::from_raw(self.raw_entry().clap_version)
    }
}

#[derive(Debug)]
pub enum PluginBundleError {
    EntryInitFailed,
    NulDescriptorPath(NulError),
    NullEntryPointer,
    InvalidUtf8Path,
    LibraryLoadingError(libloading::Error),
    IncompatibleClapVersion { plugin_version: ClapVersion },
}

impl Error for PluginBundleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PluginBundleError::NulDescriptorPath(e) => Some(e),
            PluginBundleError::LibraryLoadingError(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for PluginBundleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginBundleError::EntryInitFailed => f.write_str("Plugin entry initialization failed"),
            PluginBundleError::NulDescriptorPath(e) => {
                write!(f, "Invalid plugin descriptor path: {e}")
            }
            PluginBundleError::LibraryLoadingError(e) => {
                write!(f, "Failed to load plugin descriptor library: {e}")
            }
            PluginBundleError::InvalidUtf8Path => {
                f.write_str("Plugin descriptor path contains invalid UTF-8")
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
