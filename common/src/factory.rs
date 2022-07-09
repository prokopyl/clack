use core::ffi::c_void;
use std::ffi::CStr;
use std::ptr::NonNull;

pub mod plugin;

/// A type representing a CLAP factory.
///
/// # Safety
/// // TODO
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
