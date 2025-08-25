use clack_common::extensions::*;
use clap_sys::ext::ambisonic::{clap_host_ambisonic, clap_plugin_ambisonic, CLAP_EXT_AMBISONIC};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginAmbisonic(RawExtension<PluginExtensionSide, clap_plugin_ambisonic>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginAmbisonic {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_AMBISONIC;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct HostAmbisonic(RawExtension<HostExtensionSide, clap_host_ambisonic>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostAmbisonic {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_AMBISONIC;
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// TODO: stub
