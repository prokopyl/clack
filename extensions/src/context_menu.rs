use clack_common::extensions::{Extension, HostExtensionSide, RawExtension};
use clap_sys::ext::context_menu::*;
use std::ffi::CStr;

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

mod builder;
mod entry;
pub use builder::*;
pub use entry::*;

#[derive(Copy, Clone)]
pub struct HostContextMenu(RawExtension<HostExtensionSide, clap_host_context_menu>);

// SAFETY: The given identifiers are both valid for the context menu extension.
unsafe impl Extension for HostContextMenu {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_CONTEXT_MENU, CLAP_EXT_CONTEXT_MENU_COMPAT];
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}
