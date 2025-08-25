use clack_common::extensions::*;
use clap_sys::ext::context_menu::{
    clap_host_context_menu, clap_plugin_context_menu, CLAP_EXT_CONTEXT_MENU,
};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginContextMenu(RawExtension<PluginExtensionSide, clap_plugin_context_menu>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginContextMenu {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_CONTEXT_MENU;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct HostContextMenu(RawExtension<HostExtensionSide, clap_host_context_menu>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostContextMenu {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_CONTEXT_MENU;
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// TODO: stub
