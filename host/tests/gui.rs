//! Tests for the GUI extension.

use clack_extensions::gui::{
    GuiConfiguration, GuiError, GuiSize, PluginGui, PluginGuiImpl, Window,
};
use clack_host::prelude::*;
use clack_plugin::prelude::*;

struct TestPlugin;

impl Plugin for TestPlugin {
    type AudioProcessor<'a> = ();
    type Shared<'a> = ();
    type MainThread<'a> = TestPluginMainThread;

    fn declare_extensions(
        builder: &mut PluginExtensions<Self>,
        _shared: Option<&Self::Shared<'_>>,
    ) {
        builder.register::<PluginGui>();
    }
}

struct TestPluginMainThread;

impl PluginMainThread<'_, ()> for TestPluginMainThread {}

impl PluginGuiImpl for TestPluginMainThread {
    fn is_api_supported(&self, _configuration: GuiConfiguration) -> bool {
        true
    }

    fn get_preferred_api(&self) -> Option<GuiConfiguration<'_>> {
        None
    }

    fn create(&self, _configuration: GuiConfiguration) -> Result<(), PluginError> {
        Ok(())
    }

    fn destroy(&self) {}

    fn set_scale(&self, _scale: f64) -> Result<(), PluginError> {
        Ok(())
    }

    fn get_size(&self) -> Option<GuiSize> {
        Some(GuiSize {
            width: 100,
            height: 100,
        })
    }

    fn set_size(&self, size: GuiSize) -> Result<(), PluginError> {
        if size.width > 500 {
            Err(PluginError::Message("Size too large"))
        } else {
            Ok(())
        }
    }

    fn set_parent(&self, _window: Window) -> Result<(), PluginError> {
        Ok(())
    }

    fn set_transient(&self, _window: Window) -> Result<(), PluginError> {
        Ok(())
    }

    fn show(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn hide(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

impl DefaultPluginFactory for TestPlugin {
    fn get_descriptor() -> PluginDescriptor {
        PluginDescriptor::new("test-gui-size", "Test GUI Size")
    }

    fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
        Ok(())
    }

    fn new_main_thread(
        _host: HostMainThreadHandle,
        _shared: &(),
    ) -> Result<TestPluginMainThread, PluginError> {
        Ok(TestPluginMainThread)
    }
}

struct TestHost;

impl HostHandlers for TestHost {
    type Shared<'a> = ();
    type MainThread<'a> = ();
    type AudioProcessor<'a> = ();
}

#[test]
fn set_size_returns_error_when_plugin_fails() {
    let host_info = HostInfo::new("test", "test", "https://example.com", "1.0").unwrap();
    let entry = PluginEntry::load_from_clack::<SinglePluginEntry<TestPlugin>>(c"").unwrap();
    let mut plugin =
        PluginInstance::<TestHost>::new(|_| (), |_| (), &entry, c"test-gui-size", &host_info)
            .unwrap();

    let plugin_handle = plugin.plugin_handle();
    let gui: PluginGui = plugin_handle
        .get_extension()
        .expect("PluginGui extension not found");

    // Valid size should succeed
    let ok_result = gui.set_size(
        &plugin_handle,
        GuiSize {
            width: 200,
            height: 200,
        },
    );
    assert!(ok_result.is_ok());

    // Invalid size where plugin returns Err should fail with GuiError::SetSizeError
    let err_result = gui.set_size(
        &plugin_handle,
        GuiSize {
            width: 600,
            height: 600,
        },
    );
    assert_eq!(err_result, Err(GuiError::SetSizeError));
}
