use clack_common::entry::PluginEntryDescriptor;
use clap_sys::plugin::clap_plugin_entry;
use std::error::Error;
use std::ffi::CString;

mod descriptor;
use crate::host::PluginHost;
use crate::instance::PluginInstance;
pub use descriptor::PluginDescriptor;

pub struct PluginEntry<'a> {
    inner: &'a clap_plugin_entry,
}

impl<'a> PluginEntry<'a> {
    // TODO: handle errors properly
    /// # Safety
    /// Must only be called once for a given descriptor, else entry could be init'd multiple times
    pub unsafe fn from_raw(
        inner: &'a clap_plugin_entry,
        plugin_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        // TODO: check clap version
        let path = CString::new(plugin_path)?; // TODO: OsStr?

        // TODO: clap-sys issue: this should return bool to indicate success/failure
        (inner.init)(path.as_ptr());

        Ok(Self { inner })
    }

    // TODO: handle errors properly
    /// # Safety
    /// Must only be called once for a given descriptor, else entry could be init'd multiple times
    pub unsafe fn from_descriptor(
        desc: &'a PluginEntryDescriptor,
        plugin_path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        Self::from_raw(desc.as_raw(), plugin_path)
    }

    #[inline]
    pub fn plugin_count(&self) -> usize {
        // SAFETY: no special safety considerations
        unsafe { (self.inner.get_plugin_count)() as usize }
    }

    #[inline]
    pub fn plugin_descriptor(&self, index: usize) -> Option<PluginDescriptor<'a>> {
        // SAFETY: descriptor is guaranteed not to outlive the entry
        unsafe { (self.inner.get_plugin_descriptor)(index as u32).as_ref() }
            .map(PluginDescriptor::from_raw)
    }

    // TODO: handle errors
    #[inline]
    pub fn instantiate(&self, plugin_id: &str, host: &PluginHost) -> PluginInstance<'a> {
        PluginInstance::new(self, plugin_id, host)
    }

    #[inline]
    pub(crate) fn as_raw(&self) -> &clap_plugin_entry {
        self.inner
    }
}

impl<'a> Drop for PluginEntry<'a> {
    fn drop(&mut self) {
        // SAFETY: init() is guaranteed to have been called previously, and deinit() can only be called once.
        unsafe { (self.inner.deinit)() }
    }
}
