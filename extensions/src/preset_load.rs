use clack_common::extensions::*;
use clap_sys::ext::preset_load::{
    clap_host_preset_load, clap_plugin_preset_load, CLAP_EXT_PRESET_LOAD,
};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginPresetLoad(RawExtension<PluginExtensionSide, clap_plugin_preset_load>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginPresetLoad {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_PRESET_LOAD;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct HostPresetLoad(RawExtension<HostExtensionSide, clap_host_preset_load>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostPresetLoad {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_PRESET_LOAD;
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// TODO: stub
