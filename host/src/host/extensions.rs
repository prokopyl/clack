use crate::host::HostHandlers;
use clack_common::extensions::*;
use std::ffi::{CStr, c_void};
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A collection of all extensions supported for a given [`HostHandlers`] type.
///
/// Host can declare the different extensions they support by using the
/// [`register`](HostExtensions::register) method on this struct, during a call to
/// [`declare_extensions`](HostHandlers::declare_extensions).
pub struct HostExtensions<'a, H: ?Sized> {
    found: Option<NonNull<c_void>>,
    requested: &'a CStr,
    plugin_type: PhantomData<H>,
}

impl<'a, H: HostHandlers> HostExtensions<'a, H> {
    #[inline]
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
            .unwrap_or(core::ptr::null_mut())
    }

    /// Adds a given extension implementation to the list of extensions this plugin supports.
    pub fn register<E: ExtensionImplementation<H, ExtensionSide = HostExtensionSide>>(
        &mut self,
    ) -> &mut Self {
        if self.found.is_some() {
            return self;
        }

        if E::IDENTIFIERS.iter().any(|i| *i == self.requested) {
            self.found = Some(E::IMPLEMENTATION.as_ptr())
        }

        self
    }
}
