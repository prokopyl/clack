use clap_sys::ext::gui_win32::clap_hwnd;
use raw_window_handle::{
    AppKitHandle, HasRawWindowHandle, RawWindowHandle, Win32Handle, XlibHandle,
};
use std::ffi::c_void;

pub struct Win32Window(pub(crate) clap_hwnd);

unsafe impl HasRawWindowHandle for Win32Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = Win32Handle::empty();
        handle.hwnd = self.0;
        RawWindowHandle::Win32(handle)
    }
}

pub struct X11Window(pub(crate) ::std::os::raw::c_ulong);

unsafe impl HasRawWindowHandle for X11Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = XlibHandle::empty();
        handle.window = self.0;
        RawWindowHandle::Xlib(handle)
    }
}

pub struct CocoaWindow(pub(crate) *mut c_void);

unsafe impl HasRawWindowHandle for CocoaWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = AppKitHandle::empty();
        handle.ns_view = self.0;
        RawWindowHandle::AppKit(handle)
    }
}

pub enum AttachableWindow {
    Win32(Win32Window),
    X11(X11Window),
    Cocoa(CocoaWindow),
}

unsafe impl HasRawWindowHandle for AttachableWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        match self {
            AttachableWindow::Win32(w) => w.raw_window_handle(),
            AttachableWindow::X11(w) => w.raw_window_handle(),
            AttachableWindow::Cocoa(w) => w.raw_window_handle(),
        }
    }
}
