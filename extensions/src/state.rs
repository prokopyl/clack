use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide};
use clap_sys::ext::state::{clap_host_state, clap_plugin_state, CLAP_EXT_STATE};
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[repr(C)]
pub struct PluginState(clap_plugin_state, PhantomData<*const clap_plugin_state>);

unsafe impl Extension for PluginState {
    const IDENTIFIER: &'static CStr = CLAP_EXT_STATE;
    type ExtensionSide = PluginExtensionSide;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginState {}
unsafe impl Sync for PluginState {}

#[repr(C)]
pub struct HostState(clap_host_state, PhantomData<*const clap_host_state>);

unsafe impl Extension for HostState {
    const IDENTIFIER: &'static CStr = CLAP_EXT_STATE;
    type ExtensionSide = HostExtensionSide;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostState {}
unsafe impl Sync for HostState {}

#[derive(Copy, Clone, Debug)]
pub struct StateError {
    saving: bool,
}

impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.saving {
            f.write_str("Failed to save plugin state")
        } else {
            f.write_str("Failed to load plugin state")
        }
    }
}

impl Error for StateError {}

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;
