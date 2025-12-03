use clack_common::extensions::*;
use clap_sys::ext::configurable_audio_ports::{
    clap_plugin_configurable_audio_ports, CLAP_EXT_CONFIGURABLE_AUDIO_PORTS,
};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginConfigurableAudioPorts(
    RawExtension<PluginExtensionSide, clap_plugin_configurable_audio_ports>,
);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginConfigurableAudioPorts {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_CONFIGURABLE_AUDIO_PORTS;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// TODO: stub
