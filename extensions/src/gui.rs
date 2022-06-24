use clack_common::extensions::{Extension, HostExtension, PluginExtension};
use clack_plugin::host::HostMainThreadHandle;
use clap_sys::ext::gui::{clap_host_gui, clap_plugin_gui, CLAP_EXT_GUI};
use std::fmt::{Display, Formatter};
use std::os::raw::c_char;

/// Provide hints to host about resizing behavior
pub use clap_sys::ext::gui::clap_gui_resize_hints;
/// Handle to parent (host) window
pub use clap_sys::ext::gui::clap_window;

mod implementation;
pub use implementation::GuiApiType;
pub use implementation::PluginGui as PluginGuiImpl;

mod window;
pub use window::Window;

#[repr(C)]
pub struct PluginGui {
    inner: clap_plugin_gui,
}

unsafe impl Extension for PluginGui {
    const IDENTIFIER: *const c_char = CLAP_EXT_GUI;
    type ExtensionType = PluginExtension;
}

#[repr(C)]
pub struct HostGui {
    inner: clap_host_gui,
}

impl HostGui {
    pub fn request_resize(
        &self,
        host: &mut HostMainThreadHandle,
        width: u32,
        height: u32,
    ) -> Result<(), HostGuiError> {
        if unsafe { (self.inner.request_resize)(host.shared().as_raw(), width, height) } {
            Ok(())
        } else {
            Err(HostGuiError::ResizeError)
        }
    }
}

unsafe impl Extension for HostGui {
    const IDENTIFIER: *const c_char = CLAP_EXT_GUI;
    type ExtensionType = HostExtension;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum HostGuiError {
    ResizeError,
    ShowError,
    HideError,
}

impl Display for HostGuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HostGuiError::ResizeError => f.write_str("Request to resize plugin window failed"),
            HostGuiError::ShowError => f.write_str("Request to show plugin window failed"),
            HostGuiError::HideError => f.write_str("Request to hide plugin window failed"),
        }
    }
}

pub struct UiSize {
    pub width: u32,
    pub height: u32,
}
