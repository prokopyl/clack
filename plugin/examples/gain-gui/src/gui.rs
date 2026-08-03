//! Contains all types and implementations related to the plugin's GUI

use crate::params::GainParamsLocal;
use crate::{GainPluginMainThread, GainPluginShared, params::GainParamsShared};
use baseview::WindowHandle;
use baseview::dpi::LogicalSize;
use clack_extensions::gui::*;
use clack_plugin::prelude::*;
use egui::{CentralPanel, Context, Slider};
use egui_baseview::{EguiWindow, EguiWindowSettings};
use std::sync::Arc;

/// The EGUI application state
struct AppState {
    /// A handle to the shared params state
    shared_params: Arc<GainParamsShared>,
    /// The local state of the parameters
    local_params: GainParamsLocal,
}

impl AppState {
    /// Initializes a new [`AppState`] from the given shared params state handle
    pub fn new(shared_params: &Arc<GainParamsShared>) -> Self {
        Self {
            local_params: GainParamsLocal::new(shared_params),
            shared_params: Arc::clone(shared_params),
        }
    }
}

/// GUI state that can be accessed directly by the main thread
pub struct GainPluginGui {
    /// The handle to the baseview plugin window.
    handle: WindowHandle,
    /// The handle to the EGUI context
    egui_context: Context,
}

impl GainPluginGui {
    /// Creates a new GUI window, and embeds it into the given `parent`.
    pub fn new(parent: Window<'_, '_>, state: &GainPluginShared) -> Self {
        let settings = EguiWindowSettings::new()
            .with_tile("Gain Plugin")
            .with_size(LogicalSize::new(300, 110));

        let (tx, rx) = std::sync::mpsc::channel();

        let handle = EguiWindow::open_parented(
            &parent,
            settings,
            AppState::new(&state.params),
            move |egui_ctx, _queue, _state| tx.send(egui_ctx.clone()).unwrap(),
            |_, _, _| {},
            |egui_ctx, _queue, state: &mut AppState| {
                state.local_params.fetch_updates(&state.shared_params);

                CentralPanel::default().show(egui_ctx, |ui| {
                    ui.heading("Gain Plugin");
                    let mut value = state.local_params.get_volume();

                    let slider = ui.add(Slider::new(&mut value, 0.0..=1.0).text("gain"));

                    if slider.changed() {
                        state.local_params.set_volume(value);
                        state.local_params.push_updates(&state.shared_params);
                    };

                    state
                        .local_params
                        .has_gesture
                        .set(slider.is_pointer_button_down_on());
                    state.local_params.push_gesture(&state.shared_params);
                });
            },
        );

        let egui_context = rx.recv().unwrap();

        Self {
            handle,
            egui_context,
        }
    }

    /// Requests the UI to repaint itself, e.g. in response to events or parameter changes
    pub fn request_repaint(&self) {
        self.egui_context.request_repaint();
    }
}

impl Drop for GainPluginGui {
    fn drop(&mut self) {
        self.handle.close();
    }
}

impl<'a> PluginGuiImpl for GainPluginMainThread<'a> {
    fn is_api_supported(&self, configuration: GuiConfiguration) -> bool {
        configuration.api_type
            == GuiApiType::default_for_current_platform().expect("Unsupported platform")
            && !configuration.is_floating
    }

    fn get_preferred_api(&self) -> Option<GuiConfiguration<'_>> {
        Some(GuiConfiguration {
            api_type: GuiApiType::default_for_current_platform().expect("Unsupported platform"),
            is_floating: false,
        })
    }

    fn create(&self, configuration: GuiConfiguration) -> Result<(), PluginError> {
        if configuration.is_floating {
            return Err(PluginError::Message(
                "Invalid GUI configuration: this plugin does not support floating mode",
            ));
        }

        let supported_type =
            GuiApiType::default_for_current_platform().expect("Unsupported platform");

        if configuration.api_type != supported_type {
            return Err(PluginError::Message(
                "Invalid GUI configuration: unsupported API type",
            ));
        }

        Ok(())
    }

    fn destroy(&self) {
        let _ = self.gui.take();
    }

    fn set_scale(&self, _scale: f64) -> Result<(), PluginError> {
        Ok(())
    }

    fn get_size(&self) -> Option<GuiSize> {
        Some(GuiSize {
            width: 400,
            height: 200,
        })
    }

    fn set_size(&self, _size: GuiSize) -> Result<(), PluginError> {
        Ok(())
    }

    fn set_parent(&self, window: Window) -> Result<(), PluginError> {
        self.gui
            .replace(Some(GainPluginGui::new(window, self.shared)));
        Ok(())
    }

    fn set_transient(&self, _window: Window) -> Result<(), PluginError> {
        Ok(())
    }

    fn show(&self) -> Result<(), PluginError> {
        if let Some(gui) = &self.gui.borrow().as_ref() {
            gui.request_repaint()
        }
        Ok(())
    }

    fn hide(&self) -> Result<(), PluginError> {
        if let Some(gui) = &self.gui.borrow().as_ref() {
            gui.request_repaint()
        }

        Ok(())
    }
}
