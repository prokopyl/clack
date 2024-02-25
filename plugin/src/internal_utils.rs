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
pub(crate) unsafe fn slice_from_external_parts<'a, T>(data: *const T, len: usize) -> &'a [T] {
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
pub(crate) unsafe fn slice_from_external_parts_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T] {
    if len == 0 {
        return &mut [];
    }

    core::slice::from_raw_parts_mut(data, len)
}
