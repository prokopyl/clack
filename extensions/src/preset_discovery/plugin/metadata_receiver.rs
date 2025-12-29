use crate::utils::cstr_to_nullable_ptr;
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
        raw: *mut clap_preset_discovery_metadata_receiver,
    ) -> &'a mut Self {
        // SAFETY: This is safe to transmute as it's repr(C) and has the same memory representation
        // Other safety invariants are upheld by the caller
        unsafe { &mut *(raw as *mut Self) }
    }

    pub fn on_error(&mut self, error_code: i32, error_message: Option<&CStr>) {
        if let Some(on_error) = self.inner.on_error {
            // SAFETY: TODO
            unsafe { on_error(&self.inner, error_code, cstr_to_nullable_ptr(error_message)) }
        }
    }
}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_any!(MetadataReceiver<'static>: Send, Sync);
}
