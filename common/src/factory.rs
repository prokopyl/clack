use core::ffi::c_void;
use std::os::raw::c_char;
use std::ptr::NonNull;

pub mod plugin;

/// A type representing a CLAP factory.
///
/// # Safety
/// // TODO
pub unsafe trait Factory<'a>: Sized + 'a {
    /// The standard identifier for this factory.
    ///
    /// This MUST point to a C-style, null-terminated string.
    const IDENTIFIER: *const c_char;

    /// Returns an instance of the factory from a given factory pointer.
    ///
    /// The default implementation of this method simply casts the pointer.
    ///
    /// # Safety
    /// Callers must ensure the factory pointer points to the correct type, and also be valid for
    /// the duration of `'a`.
    #[inline]
    unsafe fn from_factory_ptr(ptr: NonNull<c_void>) -> &'a Self {
        ptr.cast().as_ref()
    }
}
