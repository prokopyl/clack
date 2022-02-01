use crate::GainPluginMainThread;
use clack_extensions::gui::free_standing::implementation::PluginFreeStandingGui;
use clack_extensions::{
    gui::attached::implementation::PluginAttachedGui, gui::attached::window::AttachableWindow,
    gui::UiSize,
};
use clack_plugin::plugin::PluginError;
use std::ffi::CStr;
use vizia::{Application, HStack, Label, WindowDescription};

impl PluginAttachedGui for GainPluginMainThread {
    fn attach(
        &mut self,
        window: AttachableWindow,
        display_name: Option<&CStr>,
    ) -> Result<(), PluginError> {
        let title = display_name
            .map(|t| t.to_string_lossy())
            .unwrap_or_else(|| "Some default title I dunno".into());

        let window = Application::new(WindowDescription::new().with_title(&title), |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "\u{e88a}");
            })
            .font_size(50.0)
            .font("material");
        })
        .open_parented(&window);

        self.open_window = Some(window);

        Ok(())
    }
}

impl PluginFreeStandingGui for GainPluginMainThread {
    fn open(&mut self) -> Result<(), PluginError> {
        let title = "Some default title I dunno";

        let window = Application::new(WindowDescription::new().with_title(&title), |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "\u{e88a}");
            })
            .font_size(50.0)
            .font("material");
        })
        .open_as_if_parented();

        self.open_window = Some(window);

        Ok(())
    }
}

impl clack_extensions::gui::implementation::PluginGui for GainPluginMainThread {
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
