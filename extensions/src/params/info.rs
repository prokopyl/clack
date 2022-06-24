use crate::utils::data_from_array_buf;
use bitflags::bitflags;
use clap_sys::ext::params::*;
use std::ffi::c_void;
use std::str::Utf8Error;

bitflags! {
    #[repr(C)]
    pub struct ParamInfoFlags: u32 {
        const IS_AUTOMATABLE = CLAP_PARAM_IS_AUTOMATABLE;
        const IS_AUTOMATABLE_PER_CHANNEL = CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL;
        const IS_AUTOMATABLE_PER_KEY = CLAP_PARAM_IS_AUTOMATABLE_PER_KEY;
        const IS_AUTOMATABLE_PER_NOTE_ID = CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID;
        const IS_AUTOMATABLE_PER_PORT = CLAP_PARAM_IS_AUTOMATABLE_PER_PORT;
        const IS_BYPASS = CLAP_PARAM_IS_BYPASS;
        const IS_HIDDEN = CLAP_PARAM_IS_HIDDEN;
        const IS_MODULATABLE = CLAP_PARAM_IS_MODULATABLE;
        const IS_MODULATABLE_PER_CHANNEL = CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL;
        const IS_MODULATABLE_PER_KEY = CLAP_PARAM_IS_MODULATABLE_PER_KEY;
        const IS_MODULATABLE_PER_NOTE_ID = CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID;
        const IS_MODULATABLE_PER_PORT = CLAP_PARAM_IS_MODULATABLE_PER_PORT;
        const IS_PERIODIC = CLAP_PARAM_IS_PERIODIC;
        const IS_READONLY = CLAP_PARAM_IS_READONLY;
        const IS_STEPPED = CLAP_PARAM_IS_STEPPED;
        const REQUIRES_PROCESS = CLAP_PARAM_REQUIRES_PROCESS;
    }
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

impl<'a> TryFrom<&'a ParamInfo> for ParamInfoData<'a> {
    type Error = Utf8Error;

    fn try_from(info: &'a ParamInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            id: info.id(),
            flags: ParamInfoFlags { bits: info.flags() },
            cookie: info.cookie(),
            name: std::str::from_utf8(info.name())?,
            module: std::str::from_utf8(info.module())?,
            min_value: info.min_value(),
            max_value: info.max_value(),
            default_value: info.default_value(),
        })
    }
}

impl<'a> TryFrom<&'a mut ParamInfo> for ParamInfoData<'a> {
    type Error = Utf8Error;

    fn try_from(info: &'a mut ParamInfo) -> Result<Self, Self::Error> {
        Self::try_from(info as &'a ParamInfo)
    }
}
