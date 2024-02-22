use super::*;
use clack_plugin::extensions::prelude::*;
use clap_sys::ext::gui::{clap_gui_resize_hints, clap_plugin_gui, clap_window};
use std::ffi::CStr;
use std::os::raw::c_char;

impl HostGui {
    /// Notify the host that the plugin window's [`GuiResizeHints`] have changed, and
    /// `get_resize_hints` should be called again.
    pub fn resize_hints_changed(&self, host: &HostHandle) {
        if let Some(resize_hints_changed) = self.inner.resize_hints_changed {
            unsafe { resize_hints_changed(host.as_raw()) }
        }
    }

    /// Requests the host to resize the parent window's client area to the given size.
    ///
    /// The host doesn't have to call the plugin's `set_size` method after accepting the request.
    ///
    /// # Errors
    ///
    /// This may return a [`GuiError::ResizeError`] if the host denied or was unable to fulfill the
    /// request.
    ///
    /// Note: as this may not be called from the main thread, a successful return value may only
    /// mean the Host acknowledged the request, and will process it asynchronously later. If the
    /// request is later found not to be able to be satisfied, then the host will call the plugin's
    /// `set_size` method to revert the operation.
    pub fn request_resize(
        &self,
        host: &HostHandle,
        width: u32,
        height: u32,
    ) -> Result<(), GuiError> {
        if unsafe {
            (self
                .inner
                .request_resize
                .ok_or(GuiError::RequestResizeError)?)(host.as_raw(), width, height)
        } {
            Ok(())
        } else {
            Err(GuiError::RequestResizeError)
        }
    }

    /// Requests the host to show the Plugin's GUI.
    ///
    /// # Errors
    ///
    /// This may return a [`GuiError::RequestShowError`] if the host denied or was unable to fulfill the
    /// request.
    pub fn request_show(&self, host: &HostHandle) -> Result<(), GuiError> {
        if unsafe { (self.inner.request_show.ok_or(GuiError::RequestShowError)?)(host.as_raw()) } {
            Ok(())
        } else {
            Err(GuiError::RequestShowError)
        }
    }

    /// Requests the host to hide the Plugin's GUI.
    ///
    /// # Errors
    ///
    /// This may return a [`GuiError::RequestHideError`] if the host denied or was unable to fulfill the
    /// request.
    pub fn request_hide(&self, host: &HostHandle) -> Result<(), GuiError> {
        if unsafe { (self.inner.request_hide.ok_or(GuiError::RequestHideError)?)(host.as_raw()) } {
            Ok(())
        } else {
            Err(GuiError::RequestHideError)
        }
    }

    /// Notifies the host that either the floating window has been closed, or that the connection to
    /// the GUI was lost.
    ///
    /// If `is_destroyed` is true, than the host must call `destroy` to acknowledge the GUI destruction.
    pub fn closed(&self, host: &HostHandle, was_destroyed: bool) {
        if let Some(closed) = self.inner.closed {
            unsafe { closed(host.as_raw(), was_destroyed) }
        }
    }
}

/// Implementation of the Plugin-side of the GUI extension.
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
pub trait PluginGuiImpl<'a> {
    /// Indicate whether a particular API is supported.
    #[allow(clippy::wrong_self_convention)] // To match the CLAP naming
    fn is_api_supported(&mut self, configuration: GuiConfiguration) -> bool;

    /// Provide a hint to the host if the plugin prefers to use an API (and/or float state).
    ///
    /// This is __only a hint__ however, and the host can still use the API of its choice and/or
    /// situate the plugin in floating or embedded state despite having called this.
    fn get_preferred_api(&mut self) -> Option<GuiConfiguration>;

    /// Create and allocate all resources needed for the GUI
    ///
    /// If `is_floating` is true, the window will not be managed by the host. The plugin can set
    /// its window to stay above the parent window via [`Self::set_transient`].
    ///
    /// If `is_floating` is false, the plugin must embed its window in the parent (host).
    fn create(&mut self, configuration: GuiConfiguration) -> Result<(), GuiError>;

    /// Free all resources associated with the GUI
    fn destroy(&mut self);

    /// Set absolute scaling factor for GUI
    ///
    /// Overrides OS settings, and should not be used if the windowing API uses logical pixels. Can
    /// be ignored if the plugin will query the OS directly and perform its own calculations.
    fn set_scale(&mut self, scale: f64) -> Result<(), GuiError> {
        Err(GuiError::SetScaleError)
    }

    /// Get current size of GUI
    fn get_size(&mut self) -> Option<GuiSize>;

    /// Tell host if GUI can be resized
    ///
    /// Only applies to embedded windows.
    fn can_resize(&mut self) -> bool {
        false
    }

    /// Provide hints on the resize-ability of the GUI
    fn get_resize_hints(&mut self) -> Option<GuiResizeHints> {
        None
    }

    /// Calculate the closest possible size for the GUI
    ///
    /// Only applies if the GUI is resizable and embedded in a parent window. Must return
    /// dimensions smaller than or equal to the requested dimensions.
    fn adjust_size(&mut self, size: GuiSize) -> Option<GuiSize> {
        None
    }

    /// Set the size of an embedded window
    fn set_size(&mut self, size: GuiSize) -> Result<(), GuiError>;

    /// Embed UI into the given parent window
    fn set_parent(&mut self, window: Window<'a>) -> Result<(), GuiError>;

    /// Receive instruction to stay above the given window
    ///
    /// Only applies to floating windows.
    fn set_transient(&mut self, window: Window<'a>) -> Result<(), GuiError>;

    /// Receive a suggested window title from the host
    ///
    /// Only applies to floating windows.
    fn suggest_title(&mut self, title: &str) {}

    /// Show the window
    fn show(&mut self) -> Result<(), GuiError>;

    /// Hide the window
    ///
    /// This should not free the resources associated with the GUI, just hide it.
    fn hide(&mut self) -> Result<(), GuiError>;
}

impl<P: Plugin> ExtensionImplementation<P> for PluginGui
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    const IMPLEMENTATION: &'static Self = &PluginGui {
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

unsafe extern "C" fn is_api_supported<P: Plugin>(
    plugin: *const clap_plugin,
    api: *const c_char,
    is_floating: bool,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin
            .main_thread()
            .as_mut()
            .is_api_supported(GuiConfiguration {
                api_type: GuiApiType(CStr::from_ptr(api)),
                is_floating,
            }))
    })
    .unwrap_or(false)
}

unsafe extern "C" fn get_preferred_api<P: Plugin>(
    plugin: *const clap_plugin,
    api: *mut *const c_char,
    floating: *mut bool,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        if api.is_null() || floating.is_null() {
            return Err(PluginWrapperError::NulPtr("get_preferred_api output"));
        }

        match plugin.main_thread().as_mut().get_preferred_api() {
            None => Ok(false),
            Some(GuiConfiguration {
                api_type,
                is_floating,
            }) => {
                *api = api_type.0.as_ptr();
                *floating = is_floating;

                Ok(true)
            }
        }
    })
    .unwrap_or(false)
}

unsafe extern "C" fn create<P: Plugin>(
    plugin: *const clap_plugin,
    api: *const c_char,
    is_floating: bool,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin
            .main_thread()
            .as_mut()
            .create(GuiConfiguration {
                api_type: GuiApiType(CStr::from_ptr(api)),
                is_floating,
            })
            .is_ok())
    })
    .unwrap_or(false)
}

unsafe extern "C" fn destroy<P: Plugin>(plugin: *const clap_plugin)
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        plugin.main_thread().as_mut().destroy();
        Ok(())
    });
}

unsafe extern "C" fn set_scale<P: Plugin>(plugin: *const clap_plugin, scale: f64) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.main_thread().as_mut().set_scale(scale).is_ok())
    })
    .unwrap_or(false)
}

unsafe extern "C" fn get_size<P: Plugin>(
    plugin: *const clap_plugin,
    width: *mut u32,
    height: *mut u32,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        if let Some(size) = plugin.main_thread().as_mut().get_size() {
            *width = size.width;
            *height = size.height;
            Ok(true)
        } else {
            *width = 0;
            *height = 0;
            Ok(false)
        }
    })
    .is_some()
}

unsafe extern "C" fn can_resize<P: Plugin>(plugin: *const clap_plugin) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.main_thread().as_mut().can_resize())
    })
    .unwrap_or(false)
}

unsafe extern "C" fn get_resize_hints<P: Plugin>(
    plugin: *const clap_plugin,
    hints: *mut clap_gui_resize_hints,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        if let Some(plugin_hints) = plugin.main_thread().as_mut().get_resize_hints() {
            *hints = plugin_hints.to_raw();
            Ok(true)
        } else {
            *hints = clap_gui_resize_hints {
                can_resize_horizontally: false,
                can_resize_vertically: false,
                preserve_aspect_ratio: false,
                aspect_ratio_width: 1,
                aspect_ratio_height: 1,
            };

            Ok(false)
        }
    })
    .unwrap_or(false)
}

unsafe extern "C" fn adjust_size<P: Plugin>(
    plugin: *const clap_plugin,
    width_adj: *mut u32,
    height_adj: *mut u32,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    if width_adj.is_null() || height_adj.is_null() {
        return false;
    }

    PluginWrapper::<P>::handle(plugin, |plugin| {
        if width_adj.is_null() || height_adj.is_null() {
            return Err(PluginWrapperError::NulPtr("adjust_size output"));
        }

        let size = GuiSize {
            width: *width_adj,
            height: *height_adj,
        };

        if let Some(best_fit) = plugin.main_thread().as_mut().adjust_size(size) {
            *width_adj = best_fit.width;
            *height_adj = best_fit.height;
            Ok(true)
        } else {
            Ok(false)
        }
    })
    .unwrap_or(false)
}

unsafe extern "C" fn set_size<P: Plugin>(
    plugin: *const clap_plugin,
    width: u32,
    height: u32,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let size = GuiSize { width, height };
        Ok(plugin.main_thread().as_mut().set_size(size))
    })
    .is_some()
}

unsafe extern "C" fn set_parent<P: Plugin>(
    plugin: *const clap_plugin,
    window: *const clap_window,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let window = window
            .as_ref()
            .ok_or(PluginWrapperError::NulPtr("clap_window"))?;

        Ok(plugin
            .main_thread()
            .as_mut()
            .set_parent(Window::from_raw(*window))
            .is_ok())
    })
    .is_some()
}

unsafe extern "C" fn set_transient<P: Plugin>(
    plugin: *const clap_plugin,
    window: *const clap_window,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let window = window
            .as_ref()
            .ok_or(PluginWrapperError::NulPtr("clap_window"))?;

        Ok(plugin
            .main_thread()
            .as_mut()
            .set_transient(Window::from_raw(*window))
            .is_ok())
    })
    .unwrap_or(false)
}

unsafe extern "C" fn suggest_title<P: Plugin>(plugin: *const clap_plugin, title: *const c_char)
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        let title = CStr::from_ptr(title)
            .to_str()
            .map_err(PluginWrapperError::StringEncoding)?;

        plugin.main_thread().as_mut().suggest_title(title);

        Ok(())
    });
}

unsafe extern "C" fn show<P: Plugin>(plugin: *const clap_plugin) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.main_thread().as_mut().show().is_ok())
    })
    .unwrap_or(false)
}

unsafe extern "C" fn hide<P: Plugin>(plugin: *const clap_plugin) -> bool
where
    for<'a> P::MainThread<'a>: PluginGuiImpl<'a>,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.main_thread().as_mut().hide().is_ok())
    })
    .unwrap_or(false)
}
