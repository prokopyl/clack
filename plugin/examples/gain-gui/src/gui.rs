//! Contains all types and implementations related to Gui window managementb
use crate::{GainPluginShared, params::GainParams};
use baseview::{Size, WindowHandle, WindowOpenOptions, WindowScalePolicy};
use egui_baseview::{
    EguiWindow, GraphicsConfig, Queue,
    egui::{self, Context, Slider},
};
use std::sync::Arc;

///
/// The type that holds the window in Clack.
///
/// This is what implements the [`HasRawWindowHandle `] trait.
#[derive(Default)]
pub struct GainPluginGui {
    /// Holds handle to plugin window.
    handle: Option<WindowHandle>,
}

impl GainPluginGui {
    /// Close Plugin window.
    pub fn close(&mut self) {
        if let Some(mut handle) = self.handle.take() {
            handle.close();
        }
    }

    /// Set parent window.
    pub fn set_parent(
        &mut self,
        parent: clack_extensions::gui::Window<'_>,
        state: &GainPluginShared,
    ) {
        let settings = WindowOpenOptions {
            title: "Gain Plugin".to_string(),
            size: Size::new(400.0, 200.0),
            scale: WindowScalePolicy::SystemScaleFactor,
            gl_config: Some(Default::default()),
        };

        self.handle = Some(EguiWindow::open_parented(
            &parent,
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
    }
}
