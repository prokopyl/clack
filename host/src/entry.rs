use clack_common::factory::Factory;
use clap_sys::entry::clap_plugin_entry;
use std::error::Error;
use std::ffi::NulError;
use std::fmt::{Display, Formatter};
use std::ptr::NonNull;

pub use clack_common::entry::*;

mod descriptor;
use crate::bundle::LoadedEntry;
pub use descriptor::PluginDescriptor;

impl<'a> PluginEntry<'a> {
    /// # Safety
    /// Must only be called once for a given descriptor, else entry could be init'd multiple times
    #[inline]
    pub unsafe fn load_from_raw(
        inner: &'a PluginEntryDescriptor,
        plugin_path: &str,
    ) -> Result<Self, PluginEntryError> {
        Ok(Self {
            inner: PluginEntryInner::FromRaw(LoadedEntry::load(inner, plugin_path)?),
        })
    }

    /// # Safety
    /// Must only be called once for a given descriptor, else entry could be init'd multiple times
    pub(crate) fn from_bundle(bundle: &'a InnerPluginBundle) -> Self {
        Self {
            inner: PluginEntryInner::FromBundle(bundle),
        }
    }

    pub fn get_factory<F: Factory>(&self) -> Option<&F> {
        let ptr = unsafe { (self.as_raw().get_factory)(F::IDENTIFIER as *const _) } as *mut _;
        NonNull::new(ptr).map(|p| unsafe { F::from_factory_ptr(p) })
    }

    #[inline]
    pub(crate) fn as_raw(&self) -> &clap_plugin_entry {
        match &self.inner {
            PluginEntryInner::FromRaw(raw) => raw.entry(),
            PluginEntryInner::FromBundle(bundle) => bundle.with_referential(|e| e.entry()),
        }
    }

    #[inline]
    pub(crate) fn loaded_bundle(&self) -> Option<&'a InnerPluginBundle> {
        match &self.inner {
            PluginEntryInner::FromRaw(_) => None,
            PluginEntryInner::FromBundle(b) => Some(b),
        }
    }

    #[inline]
    pub fn version(&self) -> ClapVersion {
        self.as_raw().clap_version
    }
}
