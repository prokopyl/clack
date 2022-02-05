use crate::{GainPluginMainThread, GainPluginShared, UiAtomics};
use clack_extensions::gui::free_standing::implementation::PluginFreeStandingGui;
use clack_extensions::{
    gui::attached::implementation::PluginAttachedGui, gui::attached::window::AttachableWindow,
    gui::UiSize,
};
use clack_plugin::plugin::PluginError;
use std::ffi::CStr;
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

impl<'a> PluginAttachedGui for GainPluginMainThread<'a> {
    fn attach(
        &mut self,
        window: AttachableWindow,
        display_name: Option<&CStr>,
    ) -> Result<(), PluginError> {
        let title = display_name
            .map(|t| t.to_string_lossy())
            .unwrap_or_else(|| "Some default title I dunno".into());
        let ui_atomics = self.shared.from_ui.clone();

        let window = Application::new(WindowDescription::new().with_title(&title), move |cx| {
            new_gui(cx, ui_atomics.clone())
        })
        .open_parented(&window);

        self.open_window = Some(window);

        Ok(())
    }
}

impl<'a> PluginFreeStandingGui for GainPluginMainThread<'a> {
    fn open(&mut self) -> Result<(), PluginError> {
        let title = "Some default title I dunno";
        let ui_atomics = self.shared.from_ui.clone();

        let window = Application::new(WindowDescription::new().with_title(title), move |cx| {
            new_gui(cx, ui_atomics.clone())
        })
        .open_as_if_parented();

        self.open_window = Some(window);

        Ok(())
    }
}

impl<'a> clack_extensions::gui::implementation::PluginGui for GainPluginMainThread<'a> {
    fn create(&mut self) -> Result<(), PluginError> {
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

    fn can_resize(&mut self) -> bool {
        false
    }

    fn set_size(&mut self, _size: UiSize) -> bool {
        false
    }

    fn show(&mut self) {}

    fn hide(&mut self) {}
}
