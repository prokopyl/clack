use core::ffi::c_void;
use std::ffi::CStr;
use std::ptr::NonNull;

pub mod plugin;

/// A base trait for plugin-side factory implementations.
///
/// # Safety
///
/// Types implementing this trait and using the default implementation of
/// [`get_raw_factory_ptr`](Factory::get_raw_factory_ptr)
/// **MUST** be `#[repr(C)]` and have the same C-FFI representation as the CLAP factory struct
/// matching the factory's [`IDENTIFIER`](Factory::IDENTIFIER).
///
/// Failure to do so will result in incorrect pointer casts and UB.
pub unsafe trait Factory {
    /// The standard identifier for this factory.
    const IDENTIFIER: &'static CStr;

    #[inline]
    fn get_raw_factory_ptr(&self) -> NonNull<c_void> {
        NonNull::from(self).cast()
    }
}
