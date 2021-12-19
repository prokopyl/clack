use crate::plugin::Plugin;
use core::ffi::c_void;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub unsafe trait Extension<'a>: Sized + 'a {
    const IDENTIFIER: *const u8;
    unsafe fn from_extension_ptr(ptr: NonNull<c_void>) -> Self;
}

pub trait ExtensionDescriptor<P> {
    const IDENTIFIER: *const u8;
    type ExtensionInterface: 'static;

    const INTERFACE: &'static Self::ExtensionInterface;
}

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

    pub fn register<E: ExtensionDescriptor<P>>(&mut self) {
        let uri = unsafe { CStr::from_ptr(E::IDENTIFIER as *const _) };
        if uri == self.requested {
            self.found = NonNull::new(E::INTERFACE as *const _ as *mut _)
        }
    }
}
