use crate::gui::GuiApiType;
use clap_sys::ext::gui::clap_window;
use raw_window_handle::{
    AppKitHandle, HasRawWindowHandle, RawWindowHandle, Win32Handle, XlibHandle,
};

pub struct Window {
    raw: clap_window,
}

impl Window {
    #[inline]
    pub(crate) unsafe fn from_raw(raw: clap_window) -> Self {
        Self { raw }
    }

    #[inline]
    pub fn api_type(&self) -> GuiApiType {
        unsafe { GuiApiType::from_ptr(self.raw.api) }
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let api_type = self.api_type();

        if api_type == GuiApiType::WIN32 {
            let mut handle = Win32Handle::empty();
            handle.hwnd = unsafe { self.raw.specific.win32 };
            RawWindowHandle::Win32(handle)
        } else if api_type == GuiApiType::COCOA {
            let mut handle = AppKitHandle::empty();
            handle.ns_view = unsafe { self.raw.specific.cocoa };
            RawWindowHandle::AppKit(handle)
        } else if api_type == GuiApiType::X11 {
            let mut handle = XlibHandle::empty();
            handle.window = unsafe { self.raw.specific.x11 };
            RawWindowHandle::Xlib(handle)
        } else {
            todo!()
        }
    }
}
