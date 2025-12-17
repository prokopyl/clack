//! Contains all types and implementations related to Gui window managementb
use crate::{GainPluginShared, params::GainParams};
use baseview::{Size, WindowHandle, WindowOpenOptions, WindowScalePolicy};
use clack_plugin::plugin::PluginError;
use egui_baseview::{
    EguiWindow, GraphicsConfig, Queue,
    egui::{self, Context, Slider},
};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::sync::Arc;

///
/// The type that holds the window in Clack.
///
/// This is what implements the [`HasRawWindowHandle `] trait.
#[derive(Default)]
pub struct GainPluginGui {
    /// Holds raw handle to parent window.
    parent: Option<RawWindowHandle>,
    /// Holds handle to plugin window.
    handle: Option<WindowHandle>,
}

unsafe impl HasRawWindowHandle for GainPluginGui {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.parent.unwrap()
    }
}

impl GainPluginGui {
    ///
    ///
    /// # Errors
    ///
    /// This function will return an error if No parent window has been provided.
    ///
    pub fn open(&mut self, state: &GainPluginShared) -> Result<(), PluginError> {
        if self.parent.is_none() {
            return Err(PluginError::Message("No parent window provided"));
        }

        let settings = WindowOpenOptions {
            title: "Gain Plugin".to_string(),
            size: Size::new(400.0, 200.0),
            scale: WindowScalePolicy::SystemScaleFactor,
            gl_config: Some(Default::default()),
        };

        self.handle = Some(EguiWindow::open_parented(
            self,
            settings,
            GraphicsConfig::default(),
            state.params.clone(),
            |_egui_ctx: &Context, _queue: &mut Queue, _state: &mut Arc<GainParams>| {},
            |egui_ctx: &Context, _queue: &mut Queue, state: &mut Arc<GainParams>| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.heading("Gain Plugin");
                    let mut value = state.get_volume();
                    if ui
                        .add(Slider::new(&mut value, 0.0..=1.0).text("gain"))
                        .changed()
                    {
                        state.set_volume(value);
                    };
                });
            },
        ));

        Ok(())
    }

    /// Close Plugin window.
    pub fn close(&mut self) {
        if let Some(handle) = self.handle.as_mut() {
            handle.close();
            self.handle = None;
        }
    }

    /// Set parent window.
    pub fn set_parent(&mut self, window: clack_extensions::gui::Window<'_>) {
        self.parent = Some(window.raw_window_handle());
    }
}
