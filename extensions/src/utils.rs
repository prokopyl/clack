#![allow(dead_code)] // Those utilities are only used in *some* extensions.

use core::ffi::c_char;
use std::ffi::CStr;

/// # Safety
///
/// Same as [`CStr::from_ptr`], except `ptr` *can* be NULL.
#[inline]
pub(crate) unsafe fn cstr_from_nullable_ptr<'a>(ptr: *const c_char) -> Option<&'a CStr> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: Upheld by caller
        unsafe { Some(CStr::from_ptr(ptr)) }
    }
}

#[inline]
pub(crate) fn cstr_to_nullable_ptr(str: Option<&CStr>) -> *const c_char {
    match str {
        Some(s) => s.as_ptr(),
        None => core::ptr::null(),
    }
}

pub(crate) fn data_from_array_buf<const N: usize>(data: &[c_char; N]) -> &[u8] {
    // SAFETY: casting from i8 to u8 is safe
    let data = unsafe { core::slice::from_raw_parts(data.as_ptr() as *const _, data.len()) };

    data.iter()
        .position(|b| *b == 0)
        .map(|pos| &data[..pos])
        .unwrap_or(data)
}

/// # Safety
///
/// The pointer must be non-null and well-aligned. However, the array doesn't need to be initialized.
/// `dst` and `value` must not overlap.
#[inline]
pub(crate) unsafe fn write_to_array_buf<const N: usize>(dst: *mut [c_char; N], value: &[u8]) {
    let max_len = core::cmp::min(N - 1, value.len()); // Space for null byte
    let value = &value[..max_len];
    // SAFETY: casting between i8 to u8 is safe
    let dst = dst.cast();
    core::ptr::copy_nonoverlapping(value.as_ptr(), dst, max_len);
    dst.add(max_len).write(0)
}

/// A safer form of [`core::slice::from_raw_parts_mut`] that returns a properly aligned slice in case
/// the length is 0.
///
/// In C it is common for empty slices to be represented using a null pointer, but this is UB in
/// Rust, as all references must be aligned and non-null.
///
/// This helper avoids that pitfall by ignoring the pointer if the length is zero.
///
/// # Safety
///
/// Same requirements as [`core::slice::from_raw_parts_mut`], except the pointer *can* be null or
/// dangling if `len == 0`.
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

#[cfg(not(test))]
#[allow(unused)]
pub(crate) use std::panic::catch_unwind as handle_panic;

#[cfg(test)]
#[inline]
#[allow(unused)]
pub(crate) fn handle_panic<F: FnOnce() -> R, R>(f: F) -> std::thread::Result<R> {
    Ok(f())
}
