use clack_common::extensions::*;
use clap_sys::ext::audio_ports_activation::{
    clap_plugin_audio_ports_activation, CLAP_EXT_AUDIO_PORTS_ACTIVATION,
};

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginAudioPortsActivation(
    RawExtension<PluginExtensionSide, clap_plugin_audio_ports_activation>,
);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginAudioPortsActivation {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_AUDIO_PORTS_ACTIVATION;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// TODO: stub
