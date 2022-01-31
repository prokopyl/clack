use bitflags::bitflags;
use clap_sys::ext::params::*;
use std::ffi::c_void;

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

fn data_from_array_buf<const N: usize>(data: &[i8; N]) -> &[u8] {
    // SAFETY: casting from i8 to u8 is safe
    let data = unsafe { ::core::slice::from_raw_parts(data.as_ptr() as *const _, data.len()) };

    data.iter()
        .position(|b| *b == 0)
        .map(|pos| &data[..pos])
        .unwrap_or(data)
}

#[repr(C)]
pub struct ParamInfo {
    pub(crate) inner: clap_param_info,
}

impl ParamInfo {
    #[inline]
    pub fn id(&self) -> u32 {
        self.inner.id
    }
    #[inline]
    pub fn flags(&self) -> u32 {
        self.inner.flags
    }
    #[inline]
    pub fn min_value(&self) -> f64 {
        self.inner.min_value
    }
    #[inline]
    pub fn max_value(&self) -> f64 {
        self.inner.max_value
    }
    #[inline]
    pub fn default_value(&self) -> f64 {
        self.inner.default_value
    }
    #[inline]
    pub fn cookie(&self) -> *mut c_void {
        self.inner.cookie
    }
    #[inline]
    pub fn module(&self) -> &[u8] {
        data_from_array_buf(&self.inner.module)
    }
    #[inline]
    pub fn name(&self) -> &[u8] {
        data_from_array_buf(&self.inner.name)
    }
}

pub struct ParamInfoData<'a> {
    pub id: u32,
    pub flags: ParamInfoFlags,
    pub cookie: *mut ::core::ffi::c_void,
    pub name: &'a str,
    pub module: &'a str,
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
}
