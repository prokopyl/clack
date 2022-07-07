use std::cmp::min;
use std::ffi::CStr;
use std::os::raw::c_char;

pub fn data_from_array_buf<const N: usize>(data: &[c_char; N]) -> &[u8] {
    // SAFETY: casting from i8 to u8 is safe
    let data = unsafe { core::slice::from_raw_parts(data.as_ptr() as *const _, data.len()) };

    data.iter()
        .position(|b| *b == 0)
        .map(|pos| &data[..pos + 1])
        .unwrap_or(data)
}

#[inline]
pub unsafe fn write_to_array_buf<const N: usize>(dst: *mut [c_char; N], value: &[u8]) {
    let max_len = min(N - 1, value.len()); // Space for null byte
    let value = &value[..max_len];
    // SAFETY: casting from i8 to u8 is safe
    let value = core::slice::from_raw_parts(value.as_ptr() as *const c_char, value.len());

    let dst = dst.cast();
    core::ptr::copy_nonoverlapping(value.as_ptr(), dst, max_len);
    dst.add(max_len).write(0)
}

pub fn from_bytes_until_nul(bytes: &[u8]) -> Result<&CStr, ()> {
    let nul_pos = bytes.iter().position(|b| *b == 0);
    match nul_pos {
        Some(nul_pos) => {
            // SAFETY: We know there is a nul byte at nul_pos, so this slice
            // (ending at the nul byte) is a well-formed C string.
            let subslice = &bytes[..nul_pos + 1];
            Ok(unsafe { CStr::from_bytes_with_nul_unchecked(subslice) })
        }
        None => Err(()),
    }
}
