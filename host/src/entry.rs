use clack_common::factory::Factory;
use clap_sys::entry::clap_plugin_entry;
use std::error::Error;
use std::ffi::{CString, NulError};
use std::fmt::{Display, Formatter};
use std::ptr::NonNull;

pub use clack_common::entry::*;

mod descriptor;
use crate::bundle::PluginBundleHandle;
pub use descriptor::PluginDescriptor;

#[derive(Clone)]
pub struct PluginEntry {
    inner: &'static clap_plugin_entry,
    pub(crate) bundle: PluginBundleHandle,
}

impl PluginEntry {
    /// # Safety
    /// Must only be called once for a given descriptor, else entry could be init'd multiple times
    #[inline]
    pub unsafe fn from_raw(
        inner: &'static PluginEntryDescriptor,
        plugin_path: &str,
    ) -> Result<Self, PluginEntryError> {
        Self::from_raw_bundle(inner, PluginBundleHandle::empty(), plugin_path)
    }

    /// # Safety
    /// Must only be called once for a given descriptor, else entry could be init'd multiple times
    pub(crate) unsafe fn from_raw_bundle(
        inner: &'static PluginEntryDescriptor,
        bundle: PluginBundleHandle,
        plugin_path: &str,
    ) -> Result<Self, PluginEntryError> {
        // TODO: check clap version
        let path = CString::new(plugin_path).map_err(PluginEntryError::NulDescriptorPath)?;

        if !(inner.init)(path.as_ptr()) {
            return Err(PluginEntryError::EntryInitFailed);
        }

        Ok(Self { inner, bundle })
    }

    pub fn get_factory<F: Factory>(&self) -> Option<&F> {
        let ptr = unsafe { (self.as_raw().get_factory)(F::IDENTIFIER as *const _) } as *mut _;
        NonNull::new(ptr).map(|p| unsafe { F::from_factory_ptr(p) })
    }

    #[inline]
    pub(crate) fn as_raw(&self) -> &clap_plugin_entry {
        self.inner
    }

    #[inline]
    pub fn version(&self) -> ClapVersion {
        self.inner.clap_version
    }
}

impl Drop for PluginEntry {
    fn drop(&mut self) {
        // SAFETY: init() is guaranteed to have been called previously, and deinit() can only be called once.
        unsafe { (self.inner.deinit)() }
    }
}

#[derive(Debug)]
pub enum PluginEntryError {
    EntryInitFailed,
    NulDescriptorPath(NulError),
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
        }
    }
}
