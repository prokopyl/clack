use clack_common::extensions::*;
use clap_sys::ext::remote_controls::{
    clap_host_remote_controls, clap_plugin_remote_controls, CLAP_EXT_REMOTE_CONTROLS,
};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginRemoteControls(RawExtension<PluginExtensionSide, clap_plugin_remote_controls>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginRemoteControls {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_REMOTE_CONTROLS;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct HostRemoteControls(RawExtension<HostExtensionSide, clap_host_remote_controls>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostRemoteControls {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_REMOTE_CONTROLS;
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// TODO: stub
