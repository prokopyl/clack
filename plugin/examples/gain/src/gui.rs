use crate::{GainPluginMainThread, UiAtomics};
use clack_extensions::gui::UiSize;
use clack_plugin::plugin::PluginError;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use vizia::*;

const STYLE: &str = r#"

    knob {
        width: 76px;
        height: 76px;
        background-color: #262a2d;
        border-radius: 38px;
        border-width: 2px;
        border-color: #363636;
    }

    knob .track {
        background-color: #ffb74d;
    }

"#;

#[derive(Debug)]
pub enum GuiEvent {
    SetGain(i32),
}

#[derive(Lens)]
pub struct GuiModel {
    #[lens(ignore)]
    ui_atomics: Arc<UiAtomics>,
    gain: i32,
}

impl Model for GuiModel {
    fn event(&mut self, _: &mut Context, event: &mut Event) {
        if let Some(app_event) = event.message.downcast() {
            match app_event {
                GuiEvent::SetGain(value) => {
                    self.gain = *value;
                    self.ui_atomics.gain.store(*value, Ordering::Relaxed)
                }
            }
        }
    }
}

fn new_gui(cx: &mut Context, ui_atomics: Arc<UiAtomics>) {
    cx.add_theme(STYLE);
    GuiModel {
        gain: 100,
        ui_atomics,
    }
    .build(cx);

    Binding::new(cx, GuiModel::gain, |cx, value| {
        let val = *value.get(cx);
        Knob::new(cx, 0.5, (val as f32) / 200.0, false).on_changing(|knob, cx| {
            cx.emit(GuiEvent::SetGain((knob.normalized_value * 200.0) as i32));
        });
    });
}

impl<'a> clack_extensions::gui::PluginGuiImpl for GainPluginMainThread<'a> {
    fn is_api_supported(
        &self,
        _api: clack_extensions::gui::GuiApiType,
        _is_floating: bool,
    ) -> Result<(), clack_plugin::prelude::PluginError> {
        Ok(())
    }

    fn get_preferred_api(&self) -> Result<(&str, bool), clack_plugin::prelude::PluginError> {
        Err(PluginError::Custom(Box::new({
            #[derive(Debug)]
            struct GetApiError;
            impl std::fmt::Display for GetApiError {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "could not get preferred api")
                }
            }
            impl std::error::Error for GetApiError {}
            GetApiError
        })))
    }

    fn set_parent(
        &mut self,
        _window: &clack_extensions::gui::clap_window,
    ) -> Result<(), clack_plugin::prelude::PluginError> {
        Ok(())
    }

    fn create(&mut self, _api: clack_extensions::gui::GuiApiType, _is_floating: bool) -> Result<(), PluginError> {
        Ok(())
    }

    fn destroy(&mut self) {
        if let Some(mut window) = self.open_window.take() {
            window.close()
        }
    }

    fn get_size(&mut self) -> Result<UiSize, PluginError> {
        Ok(UiSize {
            width: 300,
            height: 300,
        })
    }

    fn can_resize(&self) -> bool {
        false
    }

    fn get_resize_hints(
        &self,
    ) -> Result<clack_extensions::gui::clap_gui_resize_hints, clack_plugin::prelude::PluginError>
    {
        Err(clack_plugin::prelude::PluginError::CannotRescale)
    }

    fn set_size(&mut self, _size: UiSize) -> Result<(), PluginError> {
        Err(PluginError::CannotRescale)
    }

    fn show(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn hide(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
