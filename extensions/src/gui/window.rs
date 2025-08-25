use crate::gui::GuiApiType;
use clap_sys::ext::gui::*;
use core::ffi::{c_ulong, c_void, CStr};
use std::marker::PhantomData;

/// A handle to a host-provided parent window.
///
/// `'a` is the lifetime of the host-provided [`GuiApiType`] string.
///
/// # Safety
///
/// This type does enforce that the underlying window handle is still valid. Therefore, while most
/// the methods to create this type are safe, most functions that use it will be `unsafe`
/// (e.g. [`PluginGui::set_parent`](crate::gui::PluginGui::set_parent)).
///
/// It is up to the user to ensure that the underlying window object is still valid for the lifetime
/// of the plugin instance's GUI (i.e. up until [`destroy`](crate::gui::PluginGui::destroy) is called.)
#[derive(Copy, Clone)]
pub struct Window<'a> {
    raw: clap_window,
    _api_lifetime: PhantomData<GuiApiType<'a>>,
}

impl<'a> Window<'a> {
    /// # Safety
    ///
    /// Users must ensure the provided window is fully valid.
    #[cfg(feature = "clack-plugin")]
    #[inline]
    pub(crate) unsafe fn from_raw(raw: clap_window) -> Self {
        Self {
            raw,
            _api_lifetime: PhantomData,
        }
    }

    /// Returns the windowing API that is used to handle this window.
    #[inline]
    pub fn api_type(&self) -> GuiApiType<'a> {
        // SAFETY: This type ensures the function pointer is valid.
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
    #[inline]
    pub fn from_generic_ptr(api_type: GuiApiType<'a>, generic_pointer: *mut c_void) -> Self {
        Self {
            raw: clap_window {
                api: api_type.0.as_ptr(),
                specific: clap_window_handle {
                    ptr: generic_pointer,
                },
            },
            _api_lifetime: PhantomData,
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
    #[inline]
    pub fn from_win32_hwnd(hwnd: *mut c_void) -> Window<'static> {
        Window {
            raw: clap_window {
                api: GuiApiType::WIN32.0.as_ptr(),
                specific: clap_window_handle { win32: hwnd },
            },
            _api_lifetime: PhantomData,
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
    #[inline]
    pub fn from_cocoa_nsview(nsview: *mut c_void) -> Window<'static> {
        Window {
            raw: clap_window {
                api: GuiApiType::COCOA.0.as_ptr(),
                specific: clap_window_handle { cocoa: nsview },
            },
            _api_lifetime: PhantomData,
        }
    }

    /// Returns the window's handle as an X11 window handle, if this is an X11 window.
    /// Otherwise, this returns `None`.
    pub fn as_x11_handle(&self) -> Option<c_ulong> {
        if self.api_type() == GuiApiType::X11 {
            // SAFETY: We just checked this was a COCOA window
            unsafe { Some(self.raw.specific.x11) }
        } else {
            None
        }
    }

    /// Creates a [`Window`] handle from an X11 window handle.
    #[inline]
    pub fn from_x11_handle(handle: c_ulong) -> Window<'static> {
        Window {
            raw: clap_window {
                api: GuiApiType::X11.0.as_ptr(),
                specific: clap_window_handle { x11: handle },
            },
            _api_lifetime: PhantomData,
        }
    }

    /// Matches this window's GUI API to one of the standard APIs.
    ///
    /// If the value matches one of the [`WIN32`](GuiApiType::WIN32), [`COCOA`](GuiApiType::COCOA),
    /// [`X11`](GuiApiType::X11), or [`WAYLAND`](GuiApiType::WAYLAND) constants, then a window
    /// with that constant as its API type is returned. Otherwise, [`None`] is returned.
    pub fn to_standard_api_type(&self) -> Option<Window<'static>> {
        Some(Window {
            raw: clap_window {
                api: self.api_type().to_standard_api()?.0.as_ptr(),
                specific: self.raw.specific,
            },
            _api_lifetime: PhantomData,
        })
    }
}

#[cfg(feature = "raw-window-handle_05")]
const _: () = {
    use raw_window_handle_05::{
        AppKitWindowHandle, HasRawWindowHandle, RawWindowHandle, Win32WindowHandle,
        XlibWindowHandle,
    };

    // SAFETY: this type ensures the handles are valid and are consistent across calls
    unsafe impl HasRawWindowHandle for Window<'_> {
        fn raw_window_handle(&self) -> RawWindowHandle {
            let api_type = self.api_type();

            if api_type == GuiApiType::WIN32 {
                let mut handle = Win32WindowHandle::empty();
                // SAFETY: we just checked api_type matched
                handle.hwnd = unsafe { self.raw.specific.win32 };
                RawWindowHandle::Win32(handle)
            } else if api_type == GuiApiType::COCOA {
                let mut handle = AppKitWindowHandle::empty();
                // SAFETY: we just checked api_type matched
                handle.ns_view = unsafe { self.raw.specific.cocoa };
                RawWindowHandle::AppKit(handle)
            } else if api_type == GuiApiType::X11 {
                let mut handle = XlibWindowHandle::empty();
                // SAFETY: we just checked api_type matched
                handle.window = unsafe { self.raw.specific.x11 };
                RawWindowHandle::Xlib(handle)
            } else {
                panic!("Unknown GUI API type: {api_type:?}")
            }
        }
    }

    impl Window<'static> {
        /// Creates a [`Window`] from any window object implementing [`HasRawWindowHandle`].
        ///
        /// This returns [`None`] if the given window handle isn't backed by the default supported APIs.
        #[inline]
        pub fn from_raw_window<W: HasRawWindowHandle>(window: &W) -> Option<Self> {
            Self::from_raw_window_handle(window.raw_window_handle())
        }

        /// Creates a [`Window`] from a [`RawWindowHandle`].
        ///
        /// This returns [`None`] if the given window handle isn't backed by the default supported APIs.
        #[inline]
        pub fn from_raw_window_handle(handle: RawWindowHandle) -> Option<Self> {
            match handle {
                RawWindowHandle::Win32(handle) => Some(Self::from_win32_hwnd(handle.hwnd)),
                RawWindowHandle::AppKit(handle) => Some(Self::from_cocoa_nsview(handle.ns_view)),
                RawWindowHandle::Xlib(handle) => Some(Self::from_x11_handle(handle.window)),
                _ => None,
            }
        }
    }
};

#[cfg(feature = "raw-window-handle_06")]
#[allow(deprecated)]
const _: () = {
    use raw_window_handle_06::{
        AppKitWindowHandle, HandleError, HasRawWindowHandle, RawWindowHandle, Win32WindowHandle,
        XlibWindowHandle,
    };
    use std::num::NonZeroIsize;
    use std::ptr::NonNull;

    // SAFETY: The host ensures the underlying window handles are still valid
    unsafe impl HasRawWindowHandle for Window<'_> {
        fn raw_window_handle(&self) -> Result<RawWindowHandle, HandleError> {
            let api_type = self.api_type();

            let raw = if api_type == GuiApiType::WIN32 {
                RawWindowHandle::Win32(Win32WindowHandle::new(
                    // SAFETY: we just checked api_type matched
                    NonZeroIsize::new((unsafe { self.raw.specific.win32 }) as isize).unwrap(),
                ))
            } else if api_type == GuiApiType::COCOA {
                RawWindowHandle::AppKit(AppKitWindowHandle::new(
                    // SAFETY: we just checked api_type matched
                    NonNull::new(unsafe { self.raw.specific.cocoa }).unwrap(),
                ))
            } else if api_type == GuiApiType::X11 {
                // SAFETY: we just checked api_type matched
                RawWindowHandle::Xlib(XlibWindowHandle::new(unsafe { self.raw.specific.x11 }))
            } else {
                return Err(HandleError::NotSupported);
            };

            Ok(raw)
        }
    }

    impl Window<'static> {
        /// Creates a [`Window`] from any window object implementing [`HasRawWindowHandle`].
        ///
        /// This returns [`None`] if the given window handle isn't backed by the default supported APIs.
        #[inline]
        pub fn from_window<W: HasRawWindowHandle>(window: &W) -> Option<Self> {
            Self::from_window_handle(window.raw_window_handle().ok()?)
        }

        /// Creates a [`Window`] from a [`RawWindowHandle`].
        ///
        /// This returns [`None`] if the given window handle isn't backed by the default supported APIs.
        #[inline]
        pub fn from_window_handle(handle: RawWindowHandle) -> Option<Self> {
            match handle {
                RawWindowHandle::Win32(handle) => {
                    Some(Self::from_win32_hwnd(handle.hwnd.get() as *mut _))
                }
                RawWindowHandle::AppKit(handle) => {
                    Some(Self::from_cocoa_nsview(handle.ns_view.as_ptr()))
                }
                RawWindowHandle::Xlib(handle) => Some(Self::from_x11_handle(handle.window)),
                _ => None,
            }
        }
    }
};
