use clack_common::extensions::*;
use clap_sys::ext::track_info::{
    clap_host_track_info, clap_plugin_track_info, CLAP_EXT_TRACK_INFO,
};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginTrackInfo(RawExtension<PluginExtensionSide, clap_plugin_track_info>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginTrackInfo {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_TRACK_INFO;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct HostTrackInfo(RawExtension<HostExtensionSide, clap_host_track_info>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostTrackInfo {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_TRACK_INFO;
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// TODO: stub
