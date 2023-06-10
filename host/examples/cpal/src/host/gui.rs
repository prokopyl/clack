use clack_extensions::gui::{GuiApiType, GuiError, GuiSize, PluginGui, Window as ClapWindow};
use clack_host::prelude::*;
use std::error::Error;
use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

pub struct Gui<'a> {
    gui: &'a PluginGui,
    compatible_api: Option<(GuiApiType<'static>, bool)>,
    is_open: bool,
    is_resizeable: bool,
}

impl<'a> Gui<'a> {
    pub fn new(gui: &'a PluginGui, instance: &mut PluginMainThreadHandle<'a>) -> Self {
        Self {
            gui,
            compatible_api: Self::negotiate_compatible_gui(gui, instance),
            is_open: false,
            is_resizeable: false,
        }
    }

    fn negotiate_compatible_gui(
        gui: &'a PluginGui,
        plugin: &PluginMainThreadHandle,
    ) -> Option<(GuiApiType<'static>, bool)> {
        // This implementation only supports the default: Win32 on Windows, Cocoa on MacOS, X11 on Unix
        // We completely ignore the plugin's preference here: it's platform-default or nothing.
        let platform_default = GuiApiType::default_for_current_platform()?;

        if gui.is_api_supported(plugin, platform_default, false) {
            Some((platform_default, false))
        } else if gui.is_api_supported(plugin, platform_default, true) {
            Some((platform_default, true))
        } else {
            None
        }
    }

    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
        scale_factor: f64,
        plugin: &mut PluginMainThreadHandle,
    ) -> Size {
        let uses_logical_pixels = self.compatible_api.unwrap().0.uses_logical_size();

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

        if !self.is_resizeable {
            let forced_size = self.gui.get_size(plugin).unwrap_or(size);

            return self.gui_size_to_winit_size(forced_size);
        }

        let working_size = self.gui.adjust_size(plugin, size).unwrap_or(size);
        self.gui.set_size(plugin, working_size).unwrap();

        self.gui_size_to_winit_size(working_size)
    }

    pub fn gui_size_to_winit_size(&self, size: GuiSize) -> Size {
        let Some((api, _)) = self.compatible_api else { panic!("Called gui_size_to_winit_size on incompatible plugin") };
        if api.uses_logical_size() {
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
        self.compatible_api.map(|(_, floating)| floating)
    }

    pub fn open_floating(&mut self, plugin: &mut PluginMainThreadHandle) -> Result<(), GuiError> {
        let Some((api, true)) = self.compatible_api else { panic!("Called open_floating on incompatible plugin") };
        self.gui.create(plugin, api, true)?;
        self.gui.show(plugin)?;

        Ok(())
    }

    pub fn compatible_api(&self) -> Option<(GuiApiType, bool)> {
        self.compatible_api
    }

    pub fn open_embedded(
        &mut self,
        plugin: &mut PluginMainThreadHandle,
        event_loop: &EventLoopWindowTarget<()>,
    ) -> Result<Window, Box<dyn Error>> {
        let gui = self.gui;
        let Some((api, false)) = self.compatible_api else { panic!("Called open_embedded on incompatible plugin") };

        gui.create(plugin, api, false)?;

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
        gui.show(plugin)?;
        self.is_open = true;

        Ok(window)
    }

    pub fn destroy(&mut self, plugin: &mut PluginMainThreadHandle) {
        if self.is_open {
            self.gui.destroy(plugin);
            self.is_open = false;
        }
    }
}
