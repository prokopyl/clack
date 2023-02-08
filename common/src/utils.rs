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

#[inline]
pub(crate) fn handle_panic<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
    f: F,
) -> std::thread::Result<R> {
    panic::catch_unwind(f)
}

mod fixed_point;
mod version;

pub use fixed_point::*;
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
unsafe impl Sync for Cookie {}
