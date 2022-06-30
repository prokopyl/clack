use clap_sys::entry::clap_plugin_entry;
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
mod plugin_descriptor;

pub use clack_common::bundle::*;
use clack_common::version::ClapVersion;
pub use plugin_descriptor::*;

#[derive(Clone)]
pub struct PluginBundle {
    inner: Pin<Arc<EntrySource>>,
}

impl PluginBundle {
    pub fn load<P: AsRef<OsStr>>(path: P) -> Result<Self, PluginBundleError> {
        let path = path.as_ref();
        let path_str = path.to_str().ok_or(PluginBundleError::InvalidUtf8Path)?;

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
    ) -> Result<Self, PluginBundleError> {
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
                write!(f, "Invalid plugin descriptor path: {}", e)
            }
            PluginBundleError::LibraryLoadingError(e) => {
                write!(f, "Failed to load plugin descriptor library: {}", e)
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
