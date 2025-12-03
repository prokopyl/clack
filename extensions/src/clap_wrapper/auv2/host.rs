use super::PluginInfoAsAUv2;
use super::sys::*;
use clack_host::factory::FactoryPointer;
use core::ffi::{CStr, c_void};
use core::marker::PhantomData;
use core::ptr::NonNull;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PluginFactoryAsAUv2<'a> {
    inner: *mut clap_plugin_factory_as_auv2,
    _lifetime: PhantomData<&'a clap_plugin_factory_as_auv2>,
}

// SAFETY: All exposed methods of clap_plugin_factory_as_auv2 are thread-safe
unsafe impl Sync for PluginFactoryAsAUv2<'_> {}
// SAFETY: All exposed methods of clap_plugin_factory_as_auv2 are thread-safe
unsafe impl Send for PluginFactoryAsAUv2<'_> {}

// SAFETY: This takes a clap_plugin_factory_as_auv2 pointer, which matches CLAP_PLUGIN_FACTORY_INFO_AUV2
unsafe impl<'a> FactoryPointer<'a> for PluginFactoryAsAUv2<'a> {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_INFO_AUV2;

    #[inline]
    unsafe fn from_raw(raw: NonNull<c_void>) -> Self {
        Self {
            inner: raw.as_ptr().cast(),
            _lifetime: PhantomData,
        }
    }
}

impl PluginFactoryAsAUv2<'_> {
    #[inline]
    pub fn auv2_info(&self, index: u32) -> Option<PluginInfoAsAUv2> {
        // SAFETY: This type guarantees the factory pointer is valid
        let inner = unsafe { self.inner.as_ref()? };
        let get_info = inner.get_auv2_info?;
        let mut info = PluginInfoAsAUv2::empty();

        // SAFETY:
        // - This type guarantees the factory pointer is valid
        // - We got the function from the same instance the pointer points to
        // - The info buf pointer comes from a &mut reference to the above local, so it is guaranteed to be valid
        if unsafe { get_info(self.inner, index, info.as_raw_mut()) } {
            Some(info)
        } else {
            None
        }
    }
}
