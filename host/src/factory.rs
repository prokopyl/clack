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
        // SAFETY: no special safety considerations
        unsafe { (self.inner.get_plugin_count.unwrap())(&self.inner) as usize }
    }

    #[inline]
    pub fn plugin_descriptor(&self, index: usize) -> Option<PluginDescriptor<'a>> {
        // SAFETY: descriptor is guaranteed not to outlive the entry
        unsafe { (self.inner.get_plugin_descriptor.unwrap())(&self.inner, index as u32).as_ref() }
            .map(PluginDescriptor::from_raw)
    }

    pub(crate) unsafe fn instantiate(
        &self,
        plugin_id: &[u8],
        host: &clap_host,
    ) -> Option<NonNull<clap_plugin>> {
        let plugin_id = CString::new(plugin_id).ok()?;
        let ptr = NonNull::new((self.inner.create_plugin.unwrap())(
            &self.inner,
            host,
            plugin_id.as_ptr(),
        ) as *mut clap_plugin)?
        .as_ref();

        if !(ptr.init.unwrap())(ptr) {
            return Err(HostError::InstantiationFailed).unwrap();
        }

        Some(NonNull::new_unchecked(ptr as *const _ as *mut _))
    }
}
