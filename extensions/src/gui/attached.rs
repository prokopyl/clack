use clack_common::extensions::{Extension, PluginExtension};
use clap_sys::ext::gui_cocoa::{clap_plugin_gui_cocoa, CLAP_EXT_GUI_COCOA};
use clap_sys::ext::gui_win32::{clap_plugin_gui_win32, CLAP_EXT_GUI_WIN32};
use clap_sys::ext::gui_x11::{clap_plugin_gui_x11, CLAP_EXT_GUI_X11};

pub mod window;

pub struct PluginGuiWin32 {
    inner: clap_plugin_gui_win32,
}

unsafe impl Extension for PluginGuiWin32 {
    const IDENTIFIER: &'static [u8] = CLAP_EXT_GUI_WIN32;
    type ExtensionType = PluginExtension;
}

#[cfg(feature = "clack-host")]
impl PluginGuiWin32 {
    /// # Safety
    /// The window_hwnd pointer must be valid
    pub unsafe fn attach(
        &self,
        plugin: &mut clack_host::plugin::PluginMainThread,
        window_hwnd: *mut std::ffi::c_void,
    ) -> bool {
        if let Some(attach) = self.inner.attach {
            attach(plugin.as_raw(), window_hwnd)
        } else {
            false
        }
    }
}

pub struct PluginGuiCocoa {
    inner: clap_plugin_gui_cocoa,
}

unsafe impl Extension for PluginGuiCocoa {
    const IDENTIFIER: &'static [u8] = CLAP_EXT_GUI_COCOA;
    type ExtensionType = PluginExtension;
}

#[cfg(feature = "clack-host")]
impl PluginGuiCocoa {
    /// # Safety
    /// The ns_view pointer must be valid
    pub unsafe fn attach(
        &self,
        plugin: &mut clack_host::plugin::PluginMainThread,
        ns_view: *mut std::ffi::c_void,
    ) -> bool {
        if let Some(attach) = self.inner.attach {
            attach(plugin.as_raw(), ns_view)
        } else {
            false
        }
    }
}

pub struct PluginGuiX11 {
    inner: clap_plugin_gui_x11,
}

unsafe impl Extension for PluginGuiX11 {
    const IDENTIFIER: &'static [u8] = CLAP_EXT_GUI_X11;
    type ExtensionType = PluginExtension;
}

#[cfg(feature = "clack-host")]
impl PluginGuiX11 {
    /// # Safety
    /// The window_id pointer must be valid
    pub unsafe fn attach(
        &self,
        plugin: &mut clack_host::plugin::PluginMainThread,
        display_name: Option<&std::ffi::CStr>,
        window_id: u64,
    ) -> bool {
        if let Some(attach) = self.inner.attach {
            attach(
                plugin.as_raw(),
                display_name
                    .map(|s| s.as_ptr())
                    .unwrap_or(::core::ptr::null()),
                window_id,
            )
        } else {
            false
        }
    }
}

#[cfg(feature = "clack-plugin")]
pub mod implementation {
    use crate::gui::attached::window::{AttachableWindow, CocoaWindow, Win32Window, X11Window};
    use crate::gui::attached::{PluginGuiCocoa, PluginGuiWin32, PluginGuiX11};
    use clack_common::extensions::ExtensionImplementation;
    use clack_plugin::plugin::wrapper::PluginWrapper;
    use clack_plugin::plugin::{Plugin, PluginError};
    use clap_sys::ext::gui_cocoa::clap_plugin_gui_cocoa;
    use clap_sys::ext::gui_win32::{clap_hwnd, clap_plugin_gui_win32};
    use clap_sys::ext::gui_x11::clap_plugin_gui_x11;
    use clap_sys::plugin::clap_plugin;
    use std::ffi::{c_void, CStr};

    pub trait PluginAttachedGui {
        fn attach(
            &mut self,
            window: AttachableWindow,
            display_name: Option<&CStr>,
        ) -> Result<(), PluginError>;
    }

    impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginGuiWin32
    where
        P::MainThread: PluginAttachedGui,
    {
        const IMPLEMENTATION: &'static Self = &PluginGuiWin32 {
            inner: clap_plugin_gui_win32 {
                attach: Some(attach_win32::<P>),
            },
        };
    }

    impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginGuiCocoa
    where
        P::MainThread: PluginAttachedGui,
    {
        const IMPLEMENTATION: &'static Self = &PluginGuiCocoa {
            inner: clap_plugin_gui_cocoa {
                attach: Some(attach_cocoa::<P>),
            },
        };
    }

    impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginGuiX11
    where
        P::MainThread: PluginAttachedGui,
    {
        const IMPLEMENTATION: &'static Self = &PluginGuiX11 {
            inner: clap_plugin_gui_x11 {
                attach: Some(attach_x11::<P>),
            },
        };
    }

    unsafe extern "C" fn attach_win32<'a, P: Plugin<'a>>(
        plugin: *const clap_plugin,
        window: clap_hwnd,
    ) -> bool
    where
        P::MainThread: PluginAttachedGui,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            plugin
                .main_thread()
                .as_mut()
                .attach(AttachableWindow::Win32(Win32Window(window)), None)?;
            Ok(())
        })
        .is_some()
    }

    unsafe extern "C" fn attach_cocoa<'a, P: Plugin<'a>>(
        plugin: *const clap_plugin,
        ns_view: *mut c_void,
    ) -> bool
    where
        P::MainThread: PluginAttachedGui,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            plugin
                .main_thread()
                .as_mut()
                .attach(AttachableWindow::Cocoa(CocoaWindow(ns_view)), None)?;
            Ok(())
        })
        .is_some()
    }

    unsafe extern "C" fn attach_x11<'a, P: Plugin<'a>>(
        plugin: *const clap_plugin,
        display_name: *const std::os::raw::c_char,
        window: u64,
    ) -> bool
    where
        P::MainThread: PluginAttachedGui,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            let display_name = (!display_name.is_null()).then(|| CStr::from_ptr(display_name));
            plugin
                .main_thread()
                .as_mut()
                .attach(AttachableWindow::X11(X11Window(window)), display_name)?;
            Ok(())
        })
        .is_some()
    }
}
