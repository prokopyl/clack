use crate::host::{CpalHostMainThread, CpalHostShared, MainThreadMessage};
use clack_extensions::gui::{
    GuiApiType, GuiConfiguration, GuiError, GuiSize, HostGuiImpl, PluginGui, Window as ClapWindow,
};
use clack_host::prelude::*;
use std::error::Error;
use std::ffi::CStr;
use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

impl<'a> HostGuiImpl for CpalHostShared<'a> {
    fn resize_hints_changed(&self) {
        // We don't support any resize hints
    }

    fn request_resize(&self, new_size: GuiSize) -> Result<(), GuiError> {
        self.sender
            .send(MainThreadMessage::GuiRequestResized { new_size })
            .map_err(|_| GuiError::RequestResizeError)
    }

    fn request_show(&self) -> Result<(), GuiError> {
        // We never hide the window, so showing it again does nothing.
        Ok(())
    }

    fn request_hide(&self) -> Result<(), GuiError> {
        Ok(())
    }

    fn closed(&self, _was_destroyed: bool) {
        self.sender.send(MainThreadMessage::GuiClosed).unwrap();
    }
}

impl<'a> CpalHostMainThread<'a> {
    /// Request the plugin's GUI to resize to the given physical size.
    ///
    /// The scale factor is also given in case the API uses logical pixel (Cocoa on macOS).
    pub fn resize_gui(&mut self, size: PhysicalSize<u32>, scale_factor: f64) -> Size {
        let gui = self.gui.as_mut().unwrap();
        let plugin = self.plugin.as_mut().unwrap();

        let uses_logical_pixels = gui.configuration.unwrap().api_type.uses_logical_size();

        let size = if uses_logical_pixels {
            let size = size.to_logical(scale_factor);
            GuiSize {
                width: size.width,
                height: size.height,
            }
        } else {
            GuiSize {
                width: size.width,
                height: size.height,
            }
        };

        if !gui.is_resizeable {
            let forced_size = gui.plugin_gui.get_size(plugin).unwrap_or(size);

            return gui.gui_size_to_winit_size(forced_size);
        }

        let working_size = gui.plugin_gui.adjust_size(plugin, size).unwrap_or(size);
        gui.plugin_gui.set_size(plugin, working_size).unwrap();

        gui.gui_size_to_winit_size(working_size)
    }

    /// Destroys the plugin's GUI resources, if its GUI is still open.
    pub fn destroy_gui(&mut self) {
        let gui = self.gui.as_mut().unwrap();
        let plugin = self.plugin.as_mut().unwrap();

        if gui.is_open {
            gui.plugin_gui.destroy(plugin);
            gui.is_open = false;
        }
    }
}

/// Tracks a plugin's GUI state and configuration.
pub struct Gui<'a> {
    /// The plugin's GUI extension.
    plugin_gui: &'a PluginGui,
    /// The negociated GUI configuration, or None if no compatible setup could be found.
    pub configuration: Option<GuiConfiguration<'static>>,
    /// Whether or not the GUI is currently open.
    is_open: bool,
    /// Whether or not the GUI accepts to be resized.
    is_resizeable: bool,
}

impl<'a> Gui<'a> {
    /// Initializes the GUI state for a given instance
    pub fn new(plugin_gui: &'a PluginGui, instance: &mut PluginMainThreadHandle<'a>) -> Self {
        Self {
            plugin_gui,
            configuration: Self::negotiate_configuration(plugin_gui, instance),
            is_open: false,
            is_resizeable: false,
        }
    }

    /// Tries to find a compatible configuration for the given plugin's GUI.
    ///
    /// We only support the default GUI API for the platform this is compiled for, so this method
    /// only figures out if that is okay for the plugin, and whether or not is supports embedding.
    fn negotiate_configuration(
        gui: &'a PluginGui,
        plugin: &mut PluginMainThreadHandle,
    ) -> Option<GuiConfiguration<'static>> {
        // This implementation only supports the default: Win32 on Windows, Cocoa on MacOS, X11 on Unix
        // We completely ignore the plugin's preference here: it's platform-default or nothing.
        let api_type = GuiApiType::default_for_current_platform()?;
        let mut config = GuiConfiguration {
            api_type,
            is_floating: false,
        };

        if gui.is_api_supported(plugin, config) {
            Some(config)
        } else {
            config.is_floating = true;
            if gui.is_api_supported(plugin, config) {
                Some(config)
            } else {
                None
            }
        }
    }

    /// Gets a Winit-compatible GUI size from a given plugin-GUI size.
    ///
    /// This returns a Logical Size if the current platform uses logical pixels, or a Physical Size
    /// otherwise.
    pub fn gui_size_to_winit_size(&self, size: GuiSize) -> Size {
        let Some(GuiConfiguration { api_type, .. }) = self.configuration else {
            panic!("Called gui_size_to_winit_size on incompatible plugin")
        };

        if api_type.uses_logical_size() {
            LogicalSize {
                width: size.width,
                height: size.height,
            }
            .into()
        } else {
            PhysicalSize {
                width: size.width,
                height: size.height,
            }
            .into()
        }
    }

    /// Returns `true` if GUI needs to be floating, `false` if we can embed, `None` if no GUI is
    /// supported
    pub fn needs_floating(&self) -> Option<bool> {
        self.configuration
            .map(|GuiConfiguration { is_floating, .. }| is_floating)
    }

    /// Opens the plugin's GUI in floating mode.
    pub fn open_floating(&mut self, plugin: &mut PluginMainThreadHandle) -> Result<(), GuiError> {
        let Some(configuration) = self.configuration else {
            panic!("Called open_floating on incompatible plugin")
        };
        if !configuration.is_floating {
            panic!("Called open_floating on incompatible plugin")
        };

        self.plugin_gui.create(plugin, configuration)?;
        self.plugin_gui.suggest_title(
            plugin,
            CStr::from_bytes_with_nul(b"Clack CPAL plugin!\0").unwrap(),
        );
        self.plugin_gui.show(plugin)?;

        Ok(())
    }

    /// Opens the plugin's GUI in embedded mode, and embeds it in a newly created window.
    pub fn open_embedded(
        &mut self,
        plugin: &mut PluginMainThreadHandle,
        event_loop: &EventLoopWindowTarget<()>,
    ) -> Result<Window, Box<dyn Error>> {
        let gui = self.plugin_gui;
        let Some(configuration) = self.configuration else {
            panic!("Called open_embedded on incompatible plugin")
        };
        if configuration.is_floating {
            panic!("Called open_embedded on incompatible plugin")
        };

        gui.create(plugin, configuration)?;

        let initial_size = gui.get_size(plugin).unwrap_or(GuiSize {
            width: 640,
            height: 480,
        });

        self.is_resizeable = gui.can_resize(plugin);

        let window = WindowBuilder::new()
            .with_title("Clack CPAL plugin!")
            .with_inner_size(PhysicalSize {
                height: initial_size.height,
                width: initial_size.width,
            })
            .with_resizable(self.is_resizeable)
            .build(event_loop)?;

        gui.set_parent(plugin, &ClapWindow::from_window(&window).unwrap())?;
        // Some plugins don't show anything until this is called, others return an error.
        let _ = gui.show(plugin);
        self.is_open = true;

        Ok(window)
    }
}
