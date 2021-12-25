use crate::plugin::Plugin;
use core::ffi::c_void;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// # Safety
/// The IDENTIFIER must match the official identifier for the given extension, otherwise
/// the extension data could be misinterpreted, and UB could occur
pub unsafe trait Extension<'a>: Sized + 'a {
    const IDENTIFIER: *const u8;

    /// # Safety
    /// The extension pointer must be valid
    unsafe fn from_extension_ptr(ptr: NonNull<c_void>) -> Self;
}

/// # Safety
/// The IDENTIFIER must match the official identifier for the given extension, otherwise
/// the extension data could be misinterpreted, and UB could occur
pub unsafe trait ExtensionDescriptor<'a, P>: Extension<'a> {
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
