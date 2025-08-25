use clack_common::extensions::*;
use clap_sys::ext::state_context::{clap_plugin_state_context, CLAP_EXT_STATE_CONTEXT};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct PluginStateContext(RawExtension<PluginExtensionSide, clap_plugin_state_context>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginStateContext {
    const IDENTIFIER: &'static std::ffi::CStr = CLAP_EXT_STATE_CONTEXT;
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// TODO(impl): for a plugin to implement StateContext extension, it is also required to implement the State extension

// TODO: stub
