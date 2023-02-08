use crate::host::HostError;
pub use clack_common::factory::*;
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use std::ffi::CStr;
use std::ptr::NonNull;

mod plugin_descriptor;
pub use plugin_descriptor::*;

#[repr(C)]
pub struct PluginFactory {
    inner: clap_plugin_factory,
}

unsafe impl Factory for PluginFactory {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_ID;
}

impl PluginFactory {
    #[inline]
    pub fn plugin_count(&self) -> usize {
        // SAFETY: no special safety considerations
        match self.inner.get_plugin_count {
            None => 0,
            Some(count) => unsafe { count(&self.inner) as usize },
        }
    }

    #[inline]
    pub fn plugin_descriptor(&self, index: usize) -> Option<PluginDescriptor> {
        // SAFETY: descriptor is guaranteed not to outlive the entry
        unsafe { (self.inner.get_plugin_descriptor?)(&self.inner, index as u32).as_ref() }
            .map(PluginDescriptor::from_raw)
    }

    #[inline]
    pub fn plugin_descriptors(&self) -> PluginDescriptorsIter {
        PluginDescriptorsIter {
            factory: self,
            count: self.plugin_count(),
            current_index: 0,
        }
    }

    pub(crate) unsafe fn create_plugin(
        &self,
        plugin_id: &CStr,
        host: *const clap_host,
    ) -> Result<NonNull<clap_plugin>, HostError> {
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

pub struct PluginDescriptorsIter<'a> {
    factory: &'a PluginFactory,
    current_index: usize,
    count: usize,
}

impl<'a> Iterator for PluginDescriptorsIter<'a> {
    type Item = PluginDescriptor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_index >= self.count {
                return None;
            }

            let descriptor = self.factory.plugin_descriptor(self.current_index);
            self.current_index += 1;

            // Skip all none-returning indexes
            if let Some(d) = descriptor {
                return Some(d);
            }
        }
    }
}
