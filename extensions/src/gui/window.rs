use crate::gui::GuiApiType;
use clap_sys::ext::gui::{clap_window, clap_window_handle};
use raw_window_handle::{
    AppKitHandle, HasRawWindowHandle, RawWindowHandle, Win32Handle, XlibHandle,
};
use std::ffi::{c_void, CStr};

/// A host-provided parent window.
pub struct Window {
    raw: clap_window,
}

impl Window {
    #[inline]
    pub(crate) unsafe fn from_raw(raw: clap_window) -> Self {
        Self { raw }
    }

    #[inline]
    pub(crate) unsafe fn as_raw(&self) -> &clap_window {
        &self.raw
    }

    /// Returns the windowing API that is used to handle this window.
    #[inline]
    pub fn api_type(&self) -> GuiApiType {
        unsafe { GuiApiType(CStr::from_ptr(self.raw.api)) }
    }

    /// Return this Window's handle as a raw C pointer.
    ///
    /// This is useful to handle custom GUI types.
    #[inline]
    pub fn raw_ptr(&self) -> *mut c_void {
        // SAFETY: it's all always representable as a pointer
        unsafe { self.raw.specific.ptr }
    }

    /// Creates a [`Window`] from a [`RawWindowHandle`].
    ///
    /// This returns [`None`] if the given window handle isn't back by the default supported APIs.
    pub fn from_raw_window_handle(handle: RawWindowHandle) -> Option<Self> {
        match handle {
            RawWindowHandle::Xlib(handle) => Some(Self {
                raw: clap_window {
                    api: GuiApiType::X11.0.as_ptr(),
                    specific: clap_window_handle { x11: handle.window },
                },
            }),
            RawWindowHandle::Win32(handle) => Some(Self {
                raw: clap_window {
                    api: GuiApiType::WIN32.0.as_ptr(),
                    specific: clap_window_handle { win32: handle.hwnd },
                },
            }),
            RawWindowHandle::AppKit(handle) => Some(Self {
                raw: clap_window {
                    api: GuiApiType::COCOA.0.as_ptr(),
                    specific: clap_window_handle {
                        cocoa: handle.ns_view,
                    },
                },
            }),
            _ => None,
        }
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
            panic!("Unknown GUI API type: {api_type:?}")
        }
    }
}
