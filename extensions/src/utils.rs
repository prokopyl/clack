use std::cmp::min;

pub fn data_from_array_buf<const N: usize>(data: &[i8; N]) -> &[u8] {
    // SAFETY: casting from i8 to u8 is safe
    let data = unsafe { ::core::slice::from_raw_parts(data.as_ptr() as *const _, data.len()) };

    data.iter()
        .position(|b| *b == 0)
        .map(|pos| &data[..pos])
        .unwrap_or(data)
}

#[inline]
pub unsafe fn write_to_array_buf<const N: usize>(dst: *mut [i8; N], value: &str) {
    let max_len = min(N - 1, value.len()); // Space for null byte
    let value = &value.as_bytes()[..max_len];
    // SAFETY: casting from i8 to u8 is safe
    let value = ::core::slice::from_raw_parts(value.as_ptr() as *const i8, value.len());

    let dst = dst.cast();
    core::ptr::copy_nonoverlapping(value.as_ptr(), dst, max_len);
    dst.add(max_len).write(0)
}
