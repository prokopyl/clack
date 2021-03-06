use crate::{GainPluginMainThread, UiAtomics};
use clack_extensions::gui::{GuiApiType, GuiError, GuiResizeHints, GuiSize, Window};
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

impl<'a> clack_extensions::gui::PluginGuiImplementation for GainPluginMainThread<'a> {
    fn is_api_supported(&self, api: GuiApiType, _is_floating: bool) -> bool {
        api.can_provide_raw_window_handle()
    }

    fn get_preferred_api(&self) -> Option<(GuiApiType<'static>, bool)> {
        None
    }

    fn create(&mut self, _api: GuiApiType, is_floating: bool) -> Result<(), GuiError> {
        self.is_floating = is_floating;
        Ok(())
    }

    fn destroy(&mut self) {
        if let Some(mut window) = self.open_window.take() {
            window.close()
        }
    }

    fn get_size(&mut self) -> Option<GuiSize> {
        Some(GuiSize {
            width: 300,
            height: 300,
        })
    }

    fn can_resize(&self) -> bool {
        false
    }

    fn get_resize_hints(&self) -> Option<GuiResizeHints> {
        None
    }

    fn set_size(&mut self, _size: GuiSize) -> Result<(), GuiError> {
        Err(GuiError::ResizeError)
    }

    fn set_parent(&mut self, window: Window) -> Result<(), GuiError> {
        self.related_window = Some(window);
        Ok(())
    }

    fn set_transient(&mut self, window: Window) -> Result<(), GuiError> {
        self.related_window = Some(window);
        Ok(())
    }

    fn show(&mut self) -> Result<(), GuiError> {
        if self.open_window.is_some() {
            return Ok(());
        }

        let title = "Some default title I dunno";
        let ui_atomics = self.shared.from_ui.clone();

        let app = Application::new(WindowDescription::new().with_title(title), move |cx| {
            new_gui(cx, ui_atomics.clone())
        });

        self.open_window = if self.is_floating {
            Some(app.open_as_if_parented())
        } else {
            Some(app.open_parented(self.related_window.as_ref().unwrap()))
        };

        Ok(())
    }

    fn hide(&mut self) -> Result<(), GuiError> {
        if let Some(mut window) = self.open_window.take() {
            window.close()
        }

        Ok(())
    }
}
