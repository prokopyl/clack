//! Various CLAP-related utilities.

#[cfg(not(test))]
#[allow(unused)]
pub(crate) use std::panic::catch_unwind as handle_panic;

#[cfg(test)]
#[inline]
#[allow(unused)]
pub(crate) fn handle_panic<F: FnOnce() -> R, R>(f: F) -> std::thread::Result<R> {
    Ok(f())
}

mod fixed_point;
mod id;
mod version;

pub use fixed_point::*;
pub use id::ClapId;
pub use version::ClapVersion;

use std::ffi::c_void;

/// An opaque pointer for use in e.g. parameter definitions and parameter-related events.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Cookie(*mut c_void);

impl Cookie {
    #[inline]
    pub const fn empty() -> Self {
        Self(core::ptr::null_mut())
    }

    #[inline]
    pub const fn from_raw(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    #[inline]
    pub const fn as_raw(&self) -> *mut c_void {
        self.0
    }
}

impl Default for Cookie {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

// SAFETY: Cookies themselves are just pointers, which plugins have to consider as Send + Sync
unsafe impl Send for Cookie {}
// SAFETY: Cookies themselves are just pointers, which plugins have to consider as Send + Sync
unsafe impl Sync for Cookie {}

/// A safer form of [`core::slice::from_raw_parts`] that returns a properly aligned slice in case
/// the length is 0.
///
/// In C it is common for empty slices to be represented using a null pointer, but this is UB in
/// Rust, as all references must be aligned and non-null.
///
/// This helper avoids that pitfall by ignoring the pointer if the length is zero.
///
/// # Safety
///
/// Same as [`core::slice::from_raw_parts`], except the provided pointer *can* be null or
/// dangling for zero-length slices.
#[inline]
pub(crate) const unsafe fn slice_from_external_parts<'a, T>(data: *const T, len: usize) -> &'a [T] {
    if len == 0 {
        return &[];
    }

    core::slice::from_raw_parts(data, len)
}

/// Same as [`slice_from_external_parts`] but for mut slices.
///
/// # Safety
///
/// Same as [`core::slice::from_raw_parts_mut`], except the provided pointer *can* be null or
/// dangling for zero-length slices.
#[inline]
pub(crate) const unsafe fn slice_from_external_parts_mut<'a, T>(
    data: *mut T,
    len: usize,
) -> &'a mut [T] {
    if len == 0 {
        return &mut [];
    }

    core::slice::from_raw_parts_mut(data, len)
}
