use clack_common::factory::Factory;
use core::ffi::c_void;
use std::ffi::CStr;
use std::ptr::NonNull;

pub mod plugin;

/// Provides an implementation of this factory for a given type `I`.
pub trait FactoryImplementation<'a, I>: Factory<'a> + 'static {
    /// The implementation of the factory.
    const IMPLEMENTATION: &'static Self;
}

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
            .unwrap_or(::core::ptr::null_mut())
    }

    /// Adds a given factory implementation to the list of extensions this plugin entry supports.
    pub fn register<'p, F: FactoryImplementation<'p, I>, I>(&mut self) -> &mut Self {
        if self.found.is_some() {
            return self;
        }

        let uri = unsafe { CStr::from_bytes_with_nul_unchecked(F::IDENTIFIER) };
        if uri == self.requested {
            self.found = NonNull::new(F::IMPLEMENTATION as *const _ as *mut _)
        }

        self
    }
}
