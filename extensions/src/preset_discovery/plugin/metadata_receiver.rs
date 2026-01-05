use crate::preset_discovery::Flags;
use crate::utils::cstr_to_nullable_ptr;
use clack_common::utils::{Timestamp, UniversalPluginId};
use clap_sys::factory::preset_discovery::clap_preset_discovery_metadata_receiver;
use std::ffi::CStr;
use std::marker::PhantomData;

#[repr(C)]
pub struct MetadataReceiver<'a> {
    inner: clap_preset_discovery_metadata_receiver,
    // Raw pointer is here to make sure this is !Send !Sync
    lifetime: PhantomData<(&'a clap_preset_discovery_metadata_receiver, *const ())>,
}

impl MetadataReceiver<'_> {
    pub(crate) unsafe fn from_raw<'a>(
        raw: *const clap_preset_discovery_metadata_receiver,
    ) -> &'a mut Self {
        // SAFETY: This is safe to transmute as it's repr(C) and has the same memory representation
        // Other safety invariants are upheld by the caller
        unsafe { &mut *(raw as *mut Self) }
    }

    #[inline]
    pub fn on_error(&mut self, error_code: i32, error_message: Option<&CStr>) {
        if let Some(on_error) = self.inner.on_error {
            // SAFETY: TODO
            unsafe { on_error(&self.inner, error_code, cstr_to_nullable_ptr(error_message)) }
        }
    }

    #[inline]
    pub fn begin_preset(&mut self, name: Option<&CStr>, load_key: Option<&CStr>) {
        if let Some(begin_preset) = self.inner.begin_preset {
            // SAFETY: TODO
            // TODO: error
            unsafe {
                begin_preset(
                    &self.inner,
                    cstr_to_nullable_ptr(name),
                    cstr_to_nullable_ptr(load_key),
                )
            };
        }
    }

    #[inline]
    pub fn add_plugin_id(&mut self, plugin_id: UniversalPluginId) {
        if let Some(add_plugin_id) = self.inner.add_plugin_id {
            let plugin_id = plugin_id.to_raw();
            // SAFETY: TODO
            unsafe { add_plugin_id(&self.inner, &plugin_id) }
        }
    }

    #[inline]
    pub fn set_soundpack_id(&mut self, soundpack_id: &CStr) {
        if let Some(set_soundpack_id) = self.inner.set_soundpack_id {
            // SAFETY: TODO
            unsafe { set_soundpack_id(&self.inner, soundpack_id.as_ptr()) }
        }
    }

    #[inline]
    pub fn set_flags(&mut self, flags: Flags) {
        if let Some(set_flags) = self.inner.set_flags {
            // SAFETY: TODO
            unsafe { set_flags(&self.inner, flags.bits()) }
        }
    }

    #[inline]
    pub fn add_creator(&mut self, creator: &CStr) {
        if let Some(add_creator) = self.inner.add_creator {
            // SAFETY: TODO
            unsafe { add_creator(&self.inner, creator.as_ptr()) }
        }
    }

    #[inline]
    pub fn set_description(&mut self, description: &CStr) {
        if let Some(set_description) = self.inner.set_description {
            // SAFETY: TODO
            unsafe { set_description(&self.inner, description.as_ptr()) }
        }
    }

    #[inline]
    pub fn set_timestamps(
        &mut self,
        creation_time: Option<Timestamp>,
        modified_time: Option<Timestamp>,
    ) {
        if let Some(set_timestamps) = self.inner.set_timestamps {
            // SAFETY: TODO
            unsafe {
                set_timestamps(
                    &self.inner,
                    Timestamp::optional_to_raw(creation_time),
                    Timestamp::optional_to_raw(modified_time),
                )
            }
        }
    }

    #[inline]
    pub fn add_feature(&mut self, feature: &CStr) {
        if let Some(add_feature) = self.inner.add_feature {
            // SAFETY: TODO
            unsafe { add_feature(&self.inner, feature.as_ptr()) }
        }
    }

    #[inline]
    pub fn add_extra_info(&mut self, key: &CStr, value: &CStr) {
        if let Some(add_extra_info) = self.inner.add_extra_info {
            // SAFETY: TODO
            unsafe { add_extra_info(&self.inner, key.as_ptr(), value.as_ptr()) }
        }
    }
}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_any!(MetadataReceiver<'static>: Send, Sync);
}
