use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide};
use clack_common::utils::Cookie;
use clap_sys::ext::params::*;
use std::ffi::CStr;

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamRescanFlags: u32 {
        const VALUES = CLAP_PARAM_RESCAN_VALUES;
        const INFO = CLAP_PARAM_RESCAN_INFO;
        const TEXT = CLAP_PARAM_RESCAN_TEXT;
        const ALL = CLAP_PARAM_RESCAN_ALL;
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamClearFlags: u32 {
        const ALL = CLAP_PARAM_CLEAR_ALL;
        const AUTOMATIONS = CLAP_PARAM_CLEAR_AUTOMATIONS;
        const MODULATIONS = CLAP_PARAM_CLEAR_MODULATIONS;
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub struct PluginParams(clap_plugin_params);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginParams {
    const IDENTIFIER: &'static CStr = CLAP_EXT_PARAMS;
    type ExtensionSide = PluginExtensionSide;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginParams {}
// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Sync for PluginParams {}

#[repr(C)]
pub struct HostParams(clap_host_params);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostParams {
    const IDENTIFIER: &'static CStr = CLAP_EXT_PARAMS;
    type ExtensionSide = HostExtensionSide;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostParams {}
// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Sync for HostParams {}

pub struct ParamInfo<'a> {
    pub id: u32,
    pub flags: ParamInfoFlags,
    pub cookie: Cookie,
    pub name: &'a [u8],
    pub module: &'a [u8],
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
}

impl<'a> ParamInfo<'a> {
    pub fn from_raw(raw: &'a clap_param_info) -> Self {
        Self {
            id: raw.id,
            flags: ParamInfoFlags::from_bits_truncate(raw.flags),
            cookie: Cookie::from_raw(raw.cookie),
            name: crate::utils::data_from_array_buf(&raw.name),
            module: crate::utils::data_from_array_buf(&raw.module),
            min_value: raw.min_value,
            max_value: raw.max_value,
            default_value: raw.default_value,
        }
    }
}

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
