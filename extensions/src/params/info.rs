use bitflags::bitflags;
use clap_sys::ext::params::*;
use clap_sys::string_sizes::{CLAP_MODULE_SIZE, CLAP_NAME_SIZE};
use std::cmp::min;

bitflags! {
    #[repr(C)]
    pub struct ParamInfoFlags: u32 {
        const IS_STEPPED = CLAP_PARAM_IS_STEPPED;
        const IS_PER_NOTE = CLAP_PARAM_IS_PER_NOTE;
        const IS_PER_CHANNEL = CLAP_PARAM_IS_PER_CHANNEL;
        const IS_PER_PORT = CLAP_PARAM_IS_PER_PORT;
        const IS_PERIODIC = CLAP_PARAM_IS_PERIODIC;
        const IS_HIDDEN = CLAP_PARAM_IS_HIDDEN;
        // TODO: check if this works
        const IS_BYPASS = (1 << 6); // Can't use native setting since it is | with stepped
        const IS_READONLY = CLAP_PARAM_IS_READONLY;
        const IS_MODULATABLE = CLAP_PARAM_IS_MODULATABLE;
        const REQUIRES_PROCESS = CLAP_PARAM_REQUIRES_PROCESS;
    }
}

#[repr(C)]
pub struct ParamInfo {
    pub(crate) inner: clap_param_info,
}

#[inline]
fn write_to_array_buf(buf: &mut [i8], value: &str) {
    let max_len = min(buf.len() - 1, value.len());
    let value = &value.as_bytes()[..max_len];
    // SAFETY: casting from i8 to u8 is safe
    let value = unsafe { ::core::slice::from_raw_parts(value.as_ptr() as *const i8, value.len()) };

    buf[..max_len].copy_from_slice(value);
}

impl ParamInfo {
    #[inline]
    pub fn new(id: u32) -> Self {
        Self {
            inner: clap_param_info {
                id,
                flags: 0,
                cookie: ::core::ptr::null_mut(),
                name: [0; CLAP_NAME_SIZE],
                module: [0; CLAP_MODULE_SIZE],
                min_value: 0.0,
                max_value: 1.0,
                default_value: 0.0,
            },
        }
    }

    #[inline]
    pub fn with_flags(&mut self, flags: ParamInfoFlags) -> &mut Self {
        self.inner.flags = flags.bits;
        self
    }

    #[inline]
    pub fn with_default_value(&mut self, default_value: f64) -> &mut Self {
        self.inner.default_value = default_value;
        self
    }

    #[inline]
    pub fn with_value_bounds(&mut self, min_value: f64, max_value: f64) -> &mut Self {
        self.inner.min_value = min_value;
        self.inner.max_value = max_value;
        self
    }

    #[inline]
    pub fn with_name(&mut self, name: &str) -> &mut Self {
        write_to_array_buf(&mut self.inner.name, name);
        self
    }

    #[inline]
    pub fn with_module(&mut self, module: &str) -> &mut Self {
        write_to_array_buf(&mut self.inner.module, module);
        self
    }
}
