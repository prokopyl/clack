use clack_common::extensions::*;
use clap_sys::ext::surround::{clap_host_surround, clap_plugin_surround, CLAP_EXT_SURROUND};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginSurround(RawExtension<PluginExtensionSide, clap_plugin_surround>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginSurround {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_SURROUND;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct HostSurround(RawExtension<HostExtensionSide, clap_host_surround>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostSurround {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_SURROUND;
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// TODO: stub
