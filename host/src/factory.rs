use crate::entry::PluginDescriptor;
use crate::wrapper::HostError;
pub use clack_common::factory::*;
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[repr(C)]
pub struct PluginFactory<'a> {
    inner: clap_plugin_factory,
    _lifetime: PhantomData<&'a clap_plugin_factory>,
}

unsafe impl<'a> Factory<'a> for PluginFactory<'a> {
    const IDENTIFIER: &'static [u8] = CLAP_PLUGIN_FACTORY_ID;
}

impl<'a> PluginFactory<'a> {
    #[inline]
    pub fn plugin_count(&self) -> usize {
        if let Some(get_plugin_count) = self.inner.get_plugin_count {
            // SAFETY: no special safety considerations
            unsafe { get_plugin_count(&self.inner) as usize }
        } else {
            0
        }
    }

    #[inline]
    pub fn plugin_descriptor(&self, index: usize) -> Option<PluginDescriptor<'a>> {
        if let Some(get_plugin_descriptor) = self.inner.get_plugin_descriptor {
            // SAFETY: descriptor is guaranteed not to outlive the entry
            unsafe { get_plugin_descriptor(&self.inner, index as u32).as_ref() }
                .map(PluginDescriptor::from_raw)
        } else {
            None
        }
    }

    pub(crate) unsafe fn instantiate(
        &self,
        plugin_id: &[u8],
        host: &clap_host,
    ) -> Result<NonNull<clap_plugin>, HostError> {
        let plugin_id = CString::new(plugin_id).map_err(|_| HostError::PluginIdNulError)?;

        let plugin = if let Some(create_plugin) = self.inner.create_plugin {
            NonNull::new(create_plugin(&self.inner, host, plugin_id.as_ptr()) as *mut clap_plugin)
                .ok_or(HostError::PluginNotFound)?
        } else {
            return Err(HostError::InstantiationFailed);
        };

        if let Some(init) = plugin.as_ref().init {
            if !init(plugin.as_ptr()) {
                return Err(HostError::InstantiationFailed);
            }
        } else {
            return Err(HostError::InstantiationFailed);
        }

        Ok(plugin)
    }
}
