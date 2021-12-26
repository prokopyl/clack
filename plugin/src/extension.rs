use crate::plugin::Plugin;
use clap_audio_common::extensions::ExtensionDescriptor;
use core::ffi::c_void;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct ExtensionDeclarations<'a, P> {
    found: Option<NonNull<c_void>>,
    requested: &'a CStr,
    plugin_type: PhantomData<P>,
}

impl<'a, 'b, P: Plugin<'b>> ExtensionDeclarations<'a, P> {
    pub(crate) fn new(requested: &'a CStr) -> Self {
        Self {
            found: None,
            requested,
            plugin_type: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn found(&self) -> *const c_void {
        self.found
            .map(|p| p.as_ptr())
            .unwrap_or(::core::ptr::null_mut())
    }

    pub fn register<E: ExtensionDescriptor<'b, P>>(&mut self) {
        if self.found.is_some() {
            return;
        }

        let uri = unsafe { CStr::from_ptr(E::IDENTIFIER as *const _) };
        if uri == self.requested {
            self.found = NonNull::new(E::INTERFACE as *const _ as *mut _)
        }
    }
}
