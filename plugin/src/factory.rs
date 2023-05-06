use clack_common::factory::Factory;
use core::ffi::c_void;
use std::ffi::CStr;
use std::ptr::NonNull;

pub mod plugin;

pub struct PluginFactories<'a> {
    found: Option<NonNull<c_void>>,
    requested: &'a CStr,
}

impl<'a> PluginFactories<'a> {
    #[inline]
    pub(crate) fn new(requested: &'a CStr) -> Self {
        Self {
            found: None,
            requested,
        }
    }

    #[inline]
    pub(crate) fn found(&self) -> *const c_void {
        self.found
            .map(|p| p.as_ptr())
            .unwrap_or(core::ptr::null_mut())
    }

    /// Adds a given factory implementation to the list of extensions this plugin entry supports.
    pub fn register<F: Factory>(&mut self, factory: &F) -> &mut Self {
        if self.found.is_some() {
            return self;
        }

        if F::IDENTIFIER == self.requested {
            self.found = Some(NonNull::from(factory).cast())
        }

        self
    }
}
