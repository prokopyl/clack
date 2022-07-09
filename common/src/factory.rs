use core::ffi::c_void;
use std::ffi::CStr;
use std::ptr::NonNull;

/// A type representing a CLAP factory.
///
/// # Safety
///
/// Types implementing this trait and using the default implementation of [`from_factory_ptr`](Factory::from_factory_ptr)
/// **MUST** be `#[repr(C)]` and have the same C-FFI representation as the CLAP factory struct
/// matching the given [`IDENTIFIER`](Factory::IDENTIFIER).
///
/// Failure to do so will result in incorrect pointer casts and UB.
pub unsafe trait Factory: Sized {
    /// The standard identifier for this factory.
    const IDENTIFIER: &'static CStr;

    /// Returns an instance of the factory from a given factory pointer.
    ///
    /// The default implementation of this method simply casts the pointer.
    ///
    /// # Safety
    /// Callers must ensure the factory pointer points to the correct type, and also be valid for
    /// the duration of `'a`.
    #[inline]
    unsafe fn from_factory_ptr<'a>(ptr: NonNull<c_void>) -> &'a Self {
        ptr.cast().as_ref()
    }
}
