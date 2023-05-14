use std::ffi::CStr;

/// A type representing a CLAP factory.
///
/// # Safety
///
/// Types implementing this trait and using the default implementation of [`from_factory_ptr`](Factory::from_factory_ptr)
/// **MUST** be `#[repr(C)]` and have the same C-FFI representation as the CLAP factory struct
/// matching the given [`IDENTIFIER`](Factory::IDENTIFIER).
///
/// Failure to do so will result in incorrect pointer casts and UB.
pub unsafe trait Factory {
    /// The standard identifier for this factory.
    const IDENTIFIER: &'static CStr;
}
