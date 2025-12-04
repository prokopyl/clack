use super::sys::*;
use super::{PluginAsVST3, PluginInfoAsVST3, SupportedNoteExpressions};
use clack_host::factory::FactoryPointer;
use clack_host::plugin::PluginMainThreadHandle;
use core::ffi::{CStr, c_void};
use core::marker::PhantomData;
use core::ptr::NonNull;

#[derive(Copy, Clone)]
pub struct PluginFactoryAsVST3<'a> {
    inner: *mut clap_plugin_factory_as_vst3,
    _lifetime: PhantomData<&'a clap_plugin_factory_as_vst3>,
}

// SAFETY: All exposed methods of clap_plugin_factory_as_vst3 are thread-safe
unsafe impl Sync for PluginFactoryAsVST3<'_> {}
// SAFETY: All exposed methods of clap_plugin_factory_as_vst3 are thread-safe
unsafe impl Send for PluginFactoryAsVST3<'_> {}

// SAFETY: This takes a clap_plugin_factory_as_vst3 pointer, which matches CLAP_PLUGIN_FACTORY_INFO_VST3
unsafe impl<'a> FactoryPointer<'a> for PluginFactoryAsVST3<'a> {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_INFO_VST3;

    #[inline]
    unsafe fn from_raw(raw: NonNull<c_void>) -> Self {
        Self {
            inner: raw.as_ptr().cast(),
            _lifetime: PhantomData,
        }
    }
}

impl PluginFactoryAsVST3<'_> {
    #[inline]
    pub fn vst3_info(&self, index: u32) -> Option<&PluginInfoAsVST3<'_>> {
        // SAFETY: This type guarantees the factory pointer is valid
        let inner = unsafe { self.inner.as_ref()? };
        let get_info = inner.get_vst3_info?;

        // SAFETY:
        // - This type guarantees the factory pointer is valid
        // - We got the function from the same instance the pointer points to
        let ptr = unsafe { get_info(self.inner, index) };

        if ptr.is_null() {
            return None;
        }

        // SAFETY: TODO
        unsafe { Some(PluginInfoAsVST3::from_raw(ptr)) }
    }
}

impl PluginAsVST3 {
    #[inline]
    pub fn get_num_midi_channels(
        &self,
        plugin: &mut PluginMainThreadHandle<'_>,
        note_port: u32,
    ) -> u32 {
        let Some(ext) = plugin.use_extension(&self.0).get_num_midi_channels else {
            return 0;
        };

        // SAFETY: TODO
        unsafe { ext(plugin.as_raw(), note_port) }
    }

    pub fn supported_note_expressions(
        &self,
        plugin: &mut PluginMainThreadHandle<'_>,
    ) -> SupportedNoteExpressions {
        let Some(ext) = plugin.use_extension(&self.0).supported_note_expressions else {
            return SupportedNoteExpressions::empty();
        };

        // SAFETY: TODO
        let supported = unsafe { ext(plugin.as_raw()) };
        SupportedNoteExpressions::from_bits_truncate(supported)
    }
}
