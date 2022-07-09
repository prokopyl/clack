use crate::gui::{UiSize, Window};
use clack_common::extensions::ExtensionImplementation;
use clack_plugin::plugin::wrapper::{PluginWrapper, PluginWrapperError};
use clack_plugin::plugin::{Plugin, PluginError};
use clap_sys::ext::gui::{clap_gui_resize_hints, clap_plugin_gui, clap_window};
use clap_sys::plugin::clap_plugin;
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Formatter};
use std::os::raw::c_char;

type PluginResult = Result<(), PluginError>;

#[derive(Copy, Clone, Eq)]
pub struct GuiApiType<'a>(pub &'a CStr);

impl<'a> PartialEq for GuiApiType<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bytes().eq(other.0.to_bytes())
    }
}

impl<'a> GuiApiType<'a> {
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: *const c_char) -> Self {
        Self(CStr::from_ptr(ptr))
    }
}

impl GuiApiType<'static> {
    pub const WIN32: Self = Self(unsafe { CStr::from_bytes_with_nul_unchecked(b"win32\0") });
    pub const COCOA: Self = Self(unsafe { CStr::from_bytes_with_nul_unchecked(b"cocoa\0") });
    pub const X11: Self = Self(unsafe { CStr::from_bytes_with_nul_unchecked(b"x11\0") });
    pub const WAYLAND: Self = Self(unsafe { CStr::from_bytes_with_nul_unchecked(b"wayland\0") });
}

impl<'a> Debug for GuiApiType<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0.to_str() {
            Ok(s) => f.write_str(s),
            Err(_) => self.0.to_bytes().fmt(f),
        }
    }
}

/// Implement this trait for your plugin's GUI handler.
///
/// ### Typical call sequence
///
/// For floating windows:
/// 1. [set_transient][Self::set_transient]
/// 2. [suggest_title][Self::suggest_title]
///
/// For embedded windows:
/// 1. [set_scale][Self::set_scale]
/// 2. [can_resize][Self::can_resize]
/// 3. Either [set_size][Self::set_size] if resizable and host remembers previous session's size,
///    or [get_size][Self::get_size] to get initial size
/// 4. [set_parent][Self::set_parent]
#[allow(unused)]
pub trait PluginGui {
    /// Indicate whether a particular API is supported.
    fn is_api_supported(&self, api: GuiApiType, is_floating: bool) -> PluginResult;

    /// Provide a hint to the host if the plugin prefers to use an API (and/or float state).
    ///
    /// This is __only a hint__ however, and the host can still use the API of its choice and/or
    /// situate the plugin in floating or embedded state despite having called this.
    fn get_preferred_api(&self) -> Result<(&str, bool), PluginError>;

    /// Create and allocate all resources needed for the GUI
    ///
    /// If `is_floating` is true, the window will not be managed by the host. The plugin can set
    /// its window to stay above the parent window via [`Self::set_transient`].
    ///
    /// If `is_floating` is false, the plugin must embed its window in the parent (host).
    fn create(&mut self, api: GuiApiType, is_floating: bool) -> PluginResult;

    /// Free all resources associated with the GUI
    fn destroy(&mut self);

    /// Set absolute scaling factor for GUI
    ///
    /// Overrides OS settings, and should not be used if the windowing API uses logical pixels. Can
    /// be ignored if the plugin will query the OS directly and perform its own calculations.
    fn set_scale(&mut self, scale: f64) -> PluginResult {
        Err(PluginError::CannotRescale)
    }

    /// Get current size of GUI
    fn get_size(&mut self) -> Result<UiSize, PluginError>;

    /// Tell host if GUI can be resized
    ///
    /// Only applies to embedded windows.
    fn can_resize(&self) -> bool {
        false
    }

    /// Provide hints on the resize-ability of the GUI
    fn get_resize_hints(&self) -> Result<clap_gui_resize_hints, PluginError>;

    /// Calculate the closest possible size for the GUI
    ///
    /// Only applies if the GUI is resizable and embedded in a parent window. Must return
    /// dimensions smaller than or equal to the requested dimensions.
    fn adjust_size(&mut self, size: UiSize) -> Result<UiSize, PluginError> {
        Err(PluginError::CannotRescale)
    }

    /// Set the size of an embedded window
    fn set_size(&mut self, size: UiSize) -> PluginResult;

    /// Embed UI into the given parent window
    fn set_parent(&mut self, window: Window) -> PluginResult;

    /// Receive instruction to stay above the given window
    ///
    /// Only applies to floating windows.
    fn set_transient(&mut self, window: Window) -> PluginResult {
        Ok(())
    }

    /// Receive a suggested window title from the host
    ///
    /// Only applies to floating windows.
    fn suggest_title(&mut self, title: &str) {}

    /// Show the window
    fn show(&mut self) -> PluginResult;

    /// Hide the window
    ///
    /// This should not free the resources associated with the GUI, just hide it.
    fn hide(&mut self) -> PluginResult;
}

impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for super::PluginGui
where
    P::MainThread: PluginGui,
{
    const IMPLEMENTATION: &'static Self = &super::PluginGui {
        inner: clap_plugin_gui {
            is_api_supported: Some(is_api_supported::<P>),
            get_preferred_api: Some(get_preferred_api::<P>),
            create: Some(create::<P>),
            destroy: Some(destroy::<P>),
            set_scale: Some(set_scale::<P>),
            get_size: Some(get_size::<P>),
            can_resize: Some(can_resize::<P>),
            get_resize_hints: Some(get_resize_hints::<P>),
            adjust_size: Some(adjust_size::<P>),
            set_size: Some(set_size::<P>),
            set_parent: Some(set_parent::<P>),
            set_transient: Some(set_transient::<P>),
            suggest_title: Some(suggest_title::<P>),
            show: Some(show::<P>),
            hide: Some(hide::<P>),
        },
    };
}

unsafe extern "C" fn is_api_supported<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    api: *const c_char,
    is_floating: bool,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin
            .main_thread()
            .as_ref()
            .is_api_supported(GuiApiType::from_ptr(api), is_floating)
            .map_err(PluginWrapperError::Plugin)
    })
    .is_some()
}

unsafe extern "C" fn get_preferred_api<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    api: *mut *const c_char,
    is_floating: *mut bool,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        if api.is_null() || (*api).is_null() {
            return Err(PluginWrapperError::NulPtr("API name was null"));
        }
        let (preferred_api, wants_to_float) = plugin
            .main_thread()
            .as_ref()
            .get_preferred_api()
            .map_err(PluginWrapperError::Plugin)?;

        *api = CString::new(preferred_api)
            .map_err(PluginWrapperError::InvalidCString)?
            .as_ptr();
        *is_floating = wants_to_float;

        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn create<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    api: *const c_char,
    is_floating: bool,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin
            .main_thread()
            .as_mut()
            .create(GuiApiType::from_ptr(api), is_floating)
            .map_err(PluginWrapperError::Plugin)
    })
    .is_some()
}

unsafe extern "C" fn destroy<'a, P: Plugin<'a>>(plugin: *const clap_plugin)
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin.main_thread().as_mut().destroy();
        Ok(())
    });
}

unsafe extern "C" fn set_scale<'a, P: Plugin<'a>>(plugin: *const clap_plugin, scale: f64) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin
            .main_thread()
            .as_mut()
            .set_scale(scale)
            .map_err(PluginWrapperError::Plugin)
    })
    .is_some()
}

unsafe extern "C" fn get_size<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let size = plugin.main_thread().as_mut().get_size()?;
        *width = size.width;
        *height = size.height;
        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn can_resize<'a, P: Plugin<'a>>(plugin: *const clap_plugin) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.main_thread().as_ref().can_resize())
    })
    .unwrap_or(false)
}

unsafe extern "C" fn get_resize_hints<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    hints: *mut clap_gui_resize_hints,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        *hints = plugin
            .main_thread()
            .as_ref()
            .get_resize_hints()
            .map_err(PluginWrapperError::Plugin)?;
        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn adjust_size<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    width_adj: *mut u32,
    height_adj: *mut u32,
) -> bool
where
    P::MainThread: PluginGui,
{
    if width_adj.is_null() || height_adj.is_null() {
        return false;
    }

    PluginWrapper::<P>::handle(plugin, |plugin| {
        let best_fit = plugin
            .main_thread()
            .as_mut()
            .adjust_size(UiSize {
                width: *width_adj,
                height: *height_adj,
            })
            .map_err(PluginWrapperError::Plugin)?;
        *width_adj = best_fit.width;
        *height_adj = best_fit.height;
        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn set_size<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    width: u32,
    height: u32,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let size = UiSize { width, height };
        Ok(plugin.main_thread().as_mut().set_size(size))
    })
    .is_some()
}

unsafe extern "C" fn set_parent<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    window: *const clap_window,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let window = window.as_ref().ok_or(PluginWrapperError::NulPtr(
            "Null pointer provided for parent window.",
        ))?;

        plugin
            .main_thread()
            .as_mut()
            .set_parent(Window::from_raw(*window))?;

        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn set_transient<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    window: *const clap_window,
) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let window = window.as_ref().ok_or(PluginWrapperError::NulPtr(
            "Null pointer provided for transient window.",
        ))?;

        plugin
            .main_thread()
            .as_mut()
            .set_transient(Window::from_raw(*window))?;

        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn suggest_title<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    title: *const c_char,
) where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let title = CStr::from_ptr(title)
            .to_str()
            .map_err(PluginWrapperError::StringEncoding)?;

        plugin.main_thread().as_mut().suggest_title(title);

        Ok(())
    });
}

unsafe extern "C" fn show<'a, P: Plugin<'a>>(plugin: *const clap_plugin) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin.main_thread().as_mut().show()?;

        Ok(())
    })
    .is_some()
}

unsafe extern "C" fn hide<'a, P: Plugin<'a>>(plugin: *const clap_plugin) -> bool
where
    P::MainThread: PluginGui,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin.main_thread().as_mut().hide()?;

        Ok(())
    })
    .is_some()
}
