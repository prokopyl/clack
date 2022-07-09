use crate::bundle::PluginDescriptor;
use crate::host::HostError;
pub use clack_common::factory::*;
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr::NonNull;

#[repr(C)]
pub struct PluginFactory {
    inner: clap_plugin_factory,
}

unsafe impl Factory for PluginFactory {
    const IDENTIFIER: *const c_char = CLAP_PLUGIN_FACTORY_ID;
}

impl PluginFactory {
    #[inline]
    pub fn plugin_count(&self) -> usize {
        // SAFETY: no special safety considerations
        unsafe { (self.inner.get_plugin_count)(&self.inner) as usize }
    }

    #[inline]
    pub fn plugin_descriptor(&self, index: usize) -> Option<PluginDescriptor> {
        // SAFETY: descriptor is guaranteed not to outlive the entry
        unsafe { (self.inner.get_plugin_descriptor)(&self.inner, index as u32).as_ref() }
            .map(PluginDescriptor::from_raw)
    }

    pub(crate) unsafe fn instantiate(
        &self,
        plugin_id: &[u8],
        host: &clap_host,
    ) -> Result<NonNull<clap_plugin>, HostError> {
        let plugin_id = CString::new(plugin_id).map_err(|_| HostError::PluginIdNulError)?;

        let plugin = NonNull::new(
            (self.inner.create_plugin)(&self.inner, host, plugin_id.as_ptr()) as *mut clap_plugin,
        )
        .ok_or(HostError::PluginNotFound)?;

        if !(plugin.as_ref().init)(plugin.as_ptr()) {
            (plugin.as_ref().destroy)(plugin.as_ptr());

            return Err(HostError::InstantiationFailed);
        }

        Ok(plugin)
    }
}
