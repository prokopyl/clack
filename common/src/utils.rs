use std::ffi::CStr;

mod panic {
    #[cfg(not(test))]
    #[allow(unused)]
    pub use std::panic::catch_unwind;

    #[cfg(test)]
    #[inline]
    #[allow(unused)]
    pub fn catch_unwind<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
        f: F,
    ) -> std::thread::Result<R> {
        Ok(f())
    }
}

pub const fn check_cstr(bytes: &[u8]) -> &CStr {
    if bytes[bytes.len() - 1] != b'\0' {
        panic!("Invalid C String: string is not null-terminated")
    }
    unsafe { core::mem::transmute(bytes) }
}

#[inline]
pub fn handle_panic<F: FnOnce() -> R + std::panic::UnwindSafe, R>(f: F) -> std::thread::Result<R> {
    panic::catch_unwind(f)
}
