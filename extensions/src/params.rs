use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionType, PluginExtensionType};
use clap_sys::ext::params::*;
use std::ffi::CStr;

#[repr(C)]
pub struct PluginParams(clap_plugin_params);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginParams {}
unsafe impl Sync for PluginParams {}

#[repr(C)]
pub struct HostParams(clap_host_params);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostParams {}
unsafe impl Sync for HostParams {}

#[cfg(feature = "clack-plugin")]
pub mod implementation;
pub mod info;

unsafe impl Extension for PluginParams {
    const IDENTIFIER: &'static CStr = CLAP_EXT_PARAMS;
    type ExtensionType = PluginExtensionType;
}

unsafe impl Extension for HostParams {
    const IDENTIFIER: &'static CStr = CLAP_EXT_PARAMS;
    type ExtensionType = HostExtensionType;
}

bitflags! {
    #[repr(C)]
    pub struct ParamRescanFlags: u32 {
        const VALUES = CLAP_PARAM_RESCAN_VALUES;
        const INFO = CLAP_PARAM_RESCAN_INFO;
        const TEXT = CLAP_PARAM_RESCAN_TEXT;
        const ALL = CLAP_PARAM_RESCAN_ALL;
    }
}

bitflags! {
    #[repr(C)]
    pub struct ParamClearFlags: u32 {
        const ALL = CLAP_PARAM_CLEAR_ALL;
        const AUTOMATIONS = CLAP_PARAM_CLEAR_AUTOMATIONS;
        const MODULATIONS = CLAP_PARAM_CLEAR_MODULATIONS;
    }
}

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;
