use crate::gui::GuiApiType;
use clap_sys::ext::gui::*;
use core::ffi::{c_ulong, c_void, CStr};
use std::marker::PhantomData;

/// A handle to a host-provided parent window.
#[derive(Copy, Clone)]
pub struct Window<'a> {
    raw: clap_window,
    _lifetime: PhantomData<&'a c_void>,
}

impl<'a> Window<'a> {
    #[cfg(feature = "clack-plugin")]
    #[inline]
    pub(crate) unsafe fn from_raw(raw: clap_window) -> Self {
        Self {
            raw,
            _lifetime: PhantomData,
        }
    }

    /// Returns the windowing API that is used to handle this window.
    #[inline]
    pub fn api_type(&self) -> GuiApiType {
        unsafe { GuiApiType(CStr::from_ptr(self.raw.api)) }
    }

    /// Returns the window handle as a reference to the C-FFI compatible CLAP struct.
    #[inline]
    pub fn as_raw(&self) -> &clap_window {
        &self.raw
    }

    /// Return this window's handle as a generic, opaque pointer.
    ///
    /// This is useful to handle custom GUI types.
    #[inline]
    pub fn as_generic_ptr(&self) -> *mut c_void {
        // SAFETY: it's all always representable as a pointer
        unsafe { self.raw.specific.ptr }
    }

    /// Creates a [`Window`] handle from a raw `generic_pointer` to a window of a custom `api_type`.
    ///
    /// # Safety
    ///
    /// Users of this method must ensure the object `generic_pointer` points to is valid for the
    /// entire duration of `'a`.
    #[inline]
    pub unsafe fn from_generic_ptr(api_type: GuiApiType<'a>, generic_pointer: *mut c_void) -> Self {
        Self {
            raw: clap_window {
                api: api_type.0.as_ptr(),
                specific: clap_window_handle {
                    ptr: generic_pointer,
                },
            },
            _lifetime: PhantomData,
        }
    }

    /// Returns the window's handle as a Win32 `HWND`, if this is a Win32 window.
    /// Otherwise, this returns `None`.
    pub fn as_win32_hwnd(&self) -> Option<*mut c_void> {
        if self.api_type() == GuiApiType::WIN32 {
            // SAFETY: We just checked this was a WIN32 window
            unsafe { Some(self.raw.specific.win32) }
        } else {
            None
        }
    }

    /// Creates a [`Window`] handle from a Win32 `HWND`.
    ///
    /// # Safety
    ///
    /// Users of this method must ensure the given `hwnd` is valid for the
    /// entire duration of `'a`.
    #[inline]
    pub unsafe fn from_win32_hwnd(hwnd: *mut c_void) -> Self {
        Self {
            raw: clap_window {
                api: GuiApiType::WIN32.0.as_ptr(),
                specific: clap_window_handle { win32: hwnd },
            },
            _lifetime: PhantomData,
        }
    }

    /// Returns the window's handle as a pointer to Cocoa `NSView`, if this is a Cocoa window.
    /// Otherwise, this returns `None`.
    pub fn as_cocoa_nsview(&self) -> Option<*mut c_void> {
        if self.api_type() == GuiApiType::COCOA {
            // SAFETY: We just checked this was a COCOA window
            unsafe { Some(self.raw.specific.cocoa) }
        } else {
            None
        }
    }

    /// Creates a [`Window`] handle from a Cocoa `NSView`.
    ///
    /// # Safety
    ///
    /// Users of this method must ensure the given `nsview` is valid for the
    /// entire duration of `'a`.
    #[inline]
    pub unsafe fn from_cocoa_nsview(nsview: *mut c_void) -> Self {
        Self {
            raw: clap_window {
                api: GuiApiType::COCOA.0.as_ptr(),
                specific: clap_window_handle { cocoa: nsview },
            },
            _lifetime: PhantomData,
        }
    }

    /// Returns the window's handle as an X11 window handle, if this is an X11 window.
    /// Otherwise, this returns `None`.
    pub fn as_x11_handle(&self) -> Option<c_ulong> {
        if self.api_type() == GuiApiType::COCOA {
            // SAFETY: We just checked this was a COCOA window
            unsafe { Some(self.raw.specific.x11) }
        } else {
            None
        }
    }

    /// Creates a [`Window`] handle from an X11 window handle.
    ///
    /// # Safety
    ///
    /// Users of this method must ensure the given `handle` is valid for the
    /// entire duration of `'a`.
    #[inline]
    pub unsafe fn from_x11_handle(handle: c_ulong) -> Self {
        Self {
            raw: clap_window {
                api: GuiApiType::X11.0.as_ptr(),
                specific: clap_window_handle { x11: handle },
            },
            _lifetime: PhantomData,
        }
    }
}

#[cfg(feature = "raw-window-handle_05")]
const _: () = {
    use raw_window_handle_05::{
        AppKitWindowHandle, HasRawWindowHandle, HasWindowHandle, RawWindowHandle,
        Win32WindowHandle, WindowHandle, XlibWindowHandle,
    };

    impl<'a> TryFrom<WindowHandle<'a>> for Window<'a> {
        type Error = ();

        fn try_from(value: WindowHandle<'a>) -> Result<Self, Self::Error> {
            match value.raw_window_handle() {
                RawWindowHandle::Win32(handle) => unsafe { Ok(Self::from_win32_hwnd(handle.hwnd)) },
                RawWindowHandle::AppKit(handle) => unsafe {
                    Ok(Self::from_cocoa_nsview(handle.ns_view))
                },
                RawWindowHandle::Xlib(handle) => unsafe {
                    Ok(Self::from_x11_handle(handle.window))
                },
                _ => Err(()),
            }
        }
    }

    unsafe impl<'a> HasRawWindowHandle for Window<'a> {
        fn raw_window_handle(&self) -> RawWindowHandle {
            let api_type = self.api_type();

            if api_type == GuiApiType::WIN32 {
                let mut handle = Win32WindowHandle::empty();
                handle.hwnd = unsafe { self.raw.specific.win32 };
                RawWindowHandle::Win32(handle)
            } else if api_type == GuiApiType::COCOA {
                let mut handle = AppKitWindowHandle::empty();
                handle.ns_view = unsafe { self.raw.specific.cocoa };
                RawWindowHandle::AppKit(handle)
            } else if api_type == GuiApiType::X11 {
                let mut handle = XlibWindowHandle::empty();
                handle.window = unsafe { self.raw.specific.x11 };
                RawWindowHandle::Xlib(handle)
            } else {
                panic!("Unknown GUI API type: {api_type:?}")
            }
        }
    }

    impl<'a> Window<'a> {
        /// Creates a [`Window`] from any window object implementing [`HasRawWindowHandle`].
        ///
        /// This returns [`None`] if the given window handle isn't backed by the default supported APIs.
        #[inline]
        pub fn from_window<W: HasWindowHandle>(window: &'a W) -> Option<Self> {
            Self::from_window_handle(window.window_handle().ok()?)
        }

        /// Creates a [`Window`] from a [`WindowHandle`].
        ///
        /// This returns [`None`] if the given window handle isn't backed by the default supported APIs.
        #[inline]
        pub fn from_window_handle(handle: WindowHandle<'a>) -> Option<Self> {
            handle.try_into().ok()
        }
    }
};
