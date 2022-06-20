use clack_common::extensions::{Extension, PluginExtension};
use clap_sys::ext::gui::{
    clap_hwnd, clap_nsview, clap_xwnd, CLAP_WINDOW_API_COCOA, CLAP_WINDOW_API_WIN32,
    CLAP_WINDOW_API_X11,
};
use std::os::raw::c_char;

pub mod window;

pub struct PluginGuiWin32 {
    #[cfg_attr(not(feature = "clack-host"), allow(unused))]
    inner: clap_hwnd,
}

unsafe impl Extension for PluginGuiWin32 {
    const IDENTIFIER: *const i8 = CLAP_WINDOW_API_WIN32;
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
        Self::attach(self, plugin, window_hwnd)
    }
}

pub struct PluginGuiCocoa {
    #[cfg_attr(not(feature = "clack-host"), allow(unused))]
    inner: clap_nsview,
}

unsafe impl Extension for PluginGuiCocoa {
    const IDENTIFIER: *const c_char = CLAP_WINDOW_API_COCOA;
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
        Self::attach(self, plugin, ns_view)
    }
}

pub struct PluginGuiX11 {
    #[cfg_attr(not(feature = "clack-host"), allow(unused))]
    inner: clap_xwnd,
}

unsafe impl Extension for PluginGuiX11 {
    const IDENTIFIER: *const c_char = CLAP_WINDOW_API_X11;
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
        window_id: ::std::os::raw::c_ulong,
    ) -> bool {
        Self::attach(self, plugin, display_name, window_id)
    }
}

#[cfg(feature = "clack-plugin")]
pub mod implementation {
    #[cfg(feature = "clack-host")]
    use crate::gui::attached::window::{CocoaWindow, Win32Window, X11Window};
    #[cfg(feature = "clack-host")]
    use clack_plugin::plugin::wrapper::PluginWrapper;
    #[cfg(feature = "clack-host")]
    use clap_sys::{
        ext::gui::{clap_hwnd, clap_nsview, clap_xwnd},
        plugin::clap_plugin,
    };

    use crate::gui::attached::window::AttachableWindow;
    use crate::gui::attached::{PluginGuiCocoa, PluginGuiWin32, PluginGuiX11};
    use clack_common::extensions::ExtensionImplementation;
    use clack_plugin::plugin::{Plugin, PluginError};
    use std::ffi::CStr;

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
            inner: std::ptr::null_mut(),
        };
    }

    impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginGuiCocoa
    where
        P::MainThread: PluginAttachedGui,
    {
        const IMPLEMENTATION: &'static Self = &PluginGuiCocoa {
            inner: std::ptr::null_mut(),
        };
    }

    impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginGuiX11
    where
        P::MainThread: PluginAttachedGui,
    {
        const IMPLEMENTATION: &'static Self = &PluginGuiX11 { inner: 0 };
    }

    #[cfg(feature = "clack-host")]
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

    #[cfg(feature = "clack-host")]
    unsafe extern "C" fn attach_cocoa<'a, P: Plugin<'a>>(
        plugin: *const clap_plugin,
        ns_view: clap_nsview,
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

    #[cfg(feature = "clack-host")]
    unsafe extern "C" fn attach_x11<'a, P: Plugin<'a>>(
        plugin: *const clap_plugin,
        display_name: *const std::os::raw::c_char,
        window: clap_xwnd,
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
