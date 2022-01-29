pub use clack_common::entry::PluginEntryDescriptor;
use clack_common::factory::Factory;
use clap_sys::entry::clap_plugin_entry;
use std::error::Error;
use std::ffi::CString;
use std::ptr::NonNull;

mod descriptor;
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
        (inner.init.unwrap())(path.as_ptr());

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

    pub fn get_factory<F: Factory<'a>>(&self) -> Option<&'a F> {
        let ptr =
            unsafe { (self.as_raw().get_factory.unwrap())(F::IDENTIFIER.as_ptr() as *const _) }
                as *mut _;
        NonNull::new(ptr).map(|p| unsafe { F::from_factory_ptr(p) })
    }

    #[inline]
    pub(crate) fn as_raw(&self) -> &clap_plugin_entry {
        self.inner
    }
}

impl<'a> Drop for PluginEntry<'a> {
    fn drop(&mut self) {
        // SAFETY: init() is guaranteed to have been called previously, and deinit() can only be called once.
        unsafe { (self.inner.deinit.unwrap())() }
    }
}
