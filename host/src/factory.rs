//! [`PluginFactory`]

use crate::bundle::PluginDescriptor;
use crate::host::HostError;
pub use clack_common::factory::*;
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use std::ffi::{CStr, CString};
use std::ptr::NonNull;

/// Every clap library has a factory that is used to create instances of
/// its plugins. A host gets a reference to the `PluginFactory` by calling
/// [`PluginBundle::get_factory`](crate::bundle::PluginBundle::get_factory).
#[repr(C)]
pub struct PluginFactory {
    inner: clap_plugin_factory,
}

unsafe impl Factory for PluginFactory {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_ID;
}

impl PluginFactory {
    /// Returns the number of plugin types in the bundle.
    #[inline]
    pub fn plugin_count(&self) -> usize {
        // SAFETY: no special safety considerations
        match self.inner.get_plugin_count {
            None => 0,
            Some(count) => unsafe { count(&self.inner) as usize },
        }
    }

    /// Returns a [`PluginDescriptor`] for the plugin at the given index.
    ///
    /// The descriptor contains the ID, which is used to instantiate the plugin.
    #[inline]
    pub fn plugin_descriptor(&self, index: usize) -> Option<PluginDescriptor> {
        // SAFETY: descriptor is guaranteed not to outlive the entry
        unsafe { (self.inner.get_plugin_descriptor?)(&self.inner, index as u32).as_ref() }
            .map(PluginDescriptor::from_raw)
    }

    pub(crate) unsafe fn instantiate(
        &self,
        plugin_id: &[u8],
        host: &clap_host,
    ) -> Result<NonNull<clap_plugin>, HostError> {
        let plugin_id = CString::new(plugin_id).map_err(|_| HostError::PluginIdNulError)?;

        let plugin = NonNull::new((self
            .inner
            .create_plugin
            .ok_or(HostError::NullFactoryCreatePluginFunction)?)(
            &self.inner,
            host,
            plugin_id.as_ptr(),
        ) as *mut clap_plugin)
        .ok_or(HostError::PluginNotFound)?;

        if let Some(init) = plugin.as_ref().init {
            if !init(plugin.as_ptr()) {
                if let Some(destroy) = plugin.as_ref().destroy {
                    destroy(plugin.as_ptr());
                }

                return Err(HostError::InstantiationFailed);
            }
        }

        Ok(plugin)
    }
}
