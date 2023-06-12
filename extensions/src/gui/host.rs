use super::*;
use clack_host::extensions::prelude::*;

impl PluginGui {
    /// Indicate whether a particular API is supported.
    pub fn is_api_supported(
        &self,
        plugin: &PluginMainThreadHandle,
        configuration: GuiConfiguration,
    ) -> bool {
        match self.inner.is_api_supported {
            Some(is_api_supported) => unsafe {
                is_api_supported(
                    plugin.as_raw(),
                    configuration.api_type.0.as_ptr(),
                    configuration.is_floating,
                )
            },
            None => false,
        }
    }
    /// Provide a hint to the host if the plugin prefers to use an API (and/or float state).
    ///
    /// This is __only a hint__ however, and the host can still use the API of its choice and/or
    /// situate the plugin in floating or embedded state despite having called this.
    pub fn get_preferred_api(&self, plugin: &PluginMainThreadHandle) -> Option<GuiConfiguration> {
        let mut api_type = core::ptr::null();
        let mut is_floating = true;

        let success = unsafe {
            self.inner.get_preferred_api?(plugin.as_raw(), &mut api_type, &mut is_floating)
        };

        if success && !api_type.is_null() {
            let api_type = unsafe { GuiApiType(CStr::from_ptr(api_type)) };
            Some(GuiConfiguration {
                api_type,
                is_floating,
            })
        } else {
            None
        }
    }

    /// Create and allocate all resources needed for the GUI
    ///
    /// If `is_floating` is true, the window will not be managed by the host. The plugin can set
    /// its window to stay above the parent window via [`Self::set_transient`].
    ///
    /// If `is_floating` is false, the plugin must embed its window in the parent (host).
    pub fn create(
        &self,
        plugin: &mut PluginMainThreadHandle,
        configuration: GuiConfiguration,
    ) -> Result<(), GuiError> {
        let success = unsafe {
            self.inner.create.ok_or(GuiError::CreateError)?(
                plugin.as_raw(),
                configuration.api_type.0.as_ptr(),
                configuration.is_floating,
            )
        };

        match success {
            true => Ok(()),
            false => Err(GuiError::CreateError),
        }
    }

    /// Free all resources associated with the GUI
    pub fn destroy(&self, plugin: &mut PluginMainThreadHandle) {
        if let Some(destroy) = self.inner.destroy {
            unsafe { destroy(plugin.as_raw()) }
        }
    }

    /// Set absolute scaling factor for GUI
    ///
    /// Overrides OS settings, and should not be used if the windowing API uses logical pixels. Can
    /// be ignored if the plugin will query the OS directly and perform its own calculations.
    pub fn set_scale(
        &self,
        plugin: &mut PluginMainThreadHandle,
        scale: f64,
    ) -> Result<(), GuiError> {
        let success =
            unsafe { self.inner.set_scale.ok_or(GuiError::CreateError)?(plugin.as_raw(), scale) };

        match success {
            true => Ok(()),
            false => Err(GuiError::SetScaleError),
        }
    }

    /// Get current size of GUI
    pub fn get_size(&self, plugin: &PluginMainThreadHandle) -> Option<GuiSize> {
        let mut width = 0;
        let mut height = 0;

        let success = unsafe { self.inner.get_size?(plugin.as_raw(), &mut width, &mut height) };

        if success && width != 0 && height != 0 {
            Some(GuiSize { width, height })
        } else {
            None
        }
    }

    /// Tell host if GUI can be resized
    ///
    /// Only applies to embedded windows.
    pub fn can_resize(&self, plugin: &PluginMainThreadHandle) -> bool {
        if let Some(can_resize) = self.inner.can_resize {
            unsafe { can_resize(plugin.as_raw()) }
        } else {
            false
        }
    }

    /// Provide hints on the resize-ability of the GUI
    pub fn get_resize_hints(&self, plugin: &PluginMainThreadHandle) -> Option<GuiResizeHints> {
        let mut hints = clap_gui_resize_hints {
            aspect_ratio_height: u32::MAX,
            aspect_ratio_width: u32::MAX,
            can_resize_horizontally: true,
            can_resize_vertically: true,
            preserve_aspect_ratio: true,
        };

        let success = unsafe { self.inner.get_resize_hints?(plugin.as_raw(), &mut hints) };

        match success {
            true if hints.aspect_ratio_height != u32::MAX
                && hints.aspect_ratio_width != u32::MAX =>
            {
                Some(GuiResizeHints::from_raw(&hints))
            }
            _ => None,
        }
    }

    /// Calculate the closest possible size for the GUI
    ///
    /// Only applies if the GUI is resizable and embedded in a parent window. Must return
    /// dimensions smaller than or equal to the requested dimensions.
    pub fn adjust_size(
        &self,
        plugin: &mut PluginMainThreadHandle,
        size: GuiSize,
    ) -> Option<GuiSize> {
        let mut new_size = size;

        unsafe {
            self.inner.adjust_size?(plugin.as_raw(), &mut new_size.width, &mut new_size.height)
                .then_some(new_size)
        }
    }

    /// Set the size of an embedded window
    pub fn set_size(
        &self,
        plugin: &mut PluginMainThreadHandle,
        size: GuiSize,
    ) -> Result<(), GuiError> {
        let success = unsafe {
            self.inner.set_size.ok_or(GuiError::SetScaleError)?(
                plugin.as_raw(),
                size.width,
                size.height,
            )
        };

        success.then_some(()).ok_or(GuiError::SetScaleError)
    }

    /// Embed UI into the given parent window
    pub fn set_parent(
        &self,
        plugin: &mut PluginMainThreadHandle,
        window: &Window,
    ) -> Result<(), GuiError> {
        let success = unsafe {
            self.inner.set_parent.ok_or(GuiError::SetParentError)?(plugin.as_raw(), window.as_raw())
        };

        success.then_some(()).ok_or(GuiError::SetParentError)
    }

    /// Receive instruction to stay above the given window
    ///
    /// Only applies to floating windows.
    pub fn set_transient(
        &self,
        plugin: &mut PluginMainThreadHandle,
        window: Window,
    ) -> Result<(), GuiError> {
        let success = unsafe {
            self.inner.set_transient.ok_or(GuiError::SetParentError)?(
                plugin.as_raw(),
                window.as_raw(),
            )
        };

        success.then_some(()).ok_or(GuiError::SetParentError)
    }

    /// Give a suggested window title to the plugin.
    ///
    /// Only applies to floating windows.
    pub fn suggest_title(&self, plugin: &mut PluginMainThreadHandle, title: &CStr) {
        if let Some(suggest_title) = self.inner.suggest_title {
            unsafe { suggest_title(plugin.as_raw(), title.as_ptr()) }
        }
    }

    /// Show the window
    pub fn show(&self, plugin: &mut PluginMainThreadHandle) -> Result<(), GuiError> {
        unsafe { self.inner.show.ok_or(GuiError::ShowError)?(plugin.as_raw()) }
            .then_some(())
            .ok_or(GuiError::ShowError)
    }

    /// Hide the window
    ///
    /// This should not free the resources associated with the GUI, just hide it.
    pub fn hide(&self, plugin: &mut PluginMainThreadHandle) -> Result<(), GuiError> {
        unsafe { self.inner.hide.ok_or(GuiError::ShowError)?(plugin.as_raw()) }
            .then_some(())
            .ok_or(GuiError::ShowError)
    }
}

/// Implementation of the Host-side of the GUI extension.
pub trait HostGuiImpl {
    /// Notify the host that the plugin window's [`GuiResizeHints`] have changed, and
    /// `get_resize_hints` should be called again.
    fn resize_hints_changed(&self);

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
    fn request_resize(&self, new_size: GuiSize) -> Result<(), GuiError>;

    /// Requests the host to show the Plugin's GUI.
    ///
    /// # Errors
    ///
    /// This may return a [`GuiError::RequestShowError`] if the host denied or was unable to fulfill the
    /// request.
    fn request_show(&self) -> Result<(), GuiError>;

    /// Requests the host to hide the Plugin's GUI.
    ///
    /// # Errors
    ///
    /// This may return a [`GuiError::RequestHideError`] if the host denied or was unable to fulfill the
    /// request.
    fn request_hide(&self) -> Result<(), GuiError>;

    /// Notifies the host that either the floating window has been closed, or that the connection to
    /// the GUI was lost.
    ///
    /// If `is_destroyed` is true, than the host must call `destroy` to acknowledge the GUI destruction.
    fn closed(&self, was_destroyed: bool);
}

impl<H: Host> ExtensionImplementation<H> for HostGui
where
    for<'a> <H as Host>::Shared<'a>: HostGuiImpl,
{
    #[doc(hidden)]
    const IMPLEMENTATION: &'static Self = &Self {
        inner: clap_host_gui {
            resize_hints_changed: Some(resize_hints_changed::<H>),
            request_resize: Some(request_resize::<H>),
            request_show: Some(request_show::<H>),
            request_hide: Some(request_hide::<H>),
            closed: Some(closed::<H>),
        },
    };
}

unsafe extern "C" fn resize_hints_changed<H: Host>(host: *const clap_host)
where
    for<'a> <H as Host>::Shared<'a>: HostGuiImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.shared().resize_hints_changed();
        Ok(())
    });
}

unsafe extern "C" fn request_resize<H: Host>(
    host: *const clap_host,
    width: u32,
    height: u32,
) -> bool
where
    for<'a> <H as Host>::Shared<'a>: HostGuiImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        Ok(host
            .shared()
            .request_resize(GuiSize { width, height })
            .is_ok())
    })
    .unwrap_or(false)
}

unsafe extern "C" fn request_show<H: Host>(host: *const clap_host) -> bool
where
    for<'a> <H as Host>::Shared<'a>: HostGuiImpl,
{
    HostWrapper::<H>::handle(host, |host| Ok(host.shared().request_show().is_ok())).unwrap_or(false)
}

unsafe extern "C" fn request_hide<H: Host>(host: *const clap_host) -> bool
where
    for<'a> <H as Host>::Shared<'a>: HostGuiImpl,
{
    HostWrapper::<H>::handle(host, |host| Ok(host.shared().request_hide().is_ok())).unwrap_or(false)
}

unsafe extern "C" fn closed<H: Host>(host: *const clap_host, was_destroyed: bool)
where
    for<'a> <H as Host>::Shared<'a>: HostGuiImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.shared().closed(was_destroyed);
        Ok(())
    });
}
