use clack_common::entry::EntryDescriptor;
use clack_plugin::clack_entry;
use clack_plugin::prelude::*;
use clap_sys::factory::plugin_factory::CLAP_PLUGIN_FACTORY_ID;

struct MyPlugin;

impl Plugin for MyPlugin {
    type AudioProcessor<'a> = ();
    type Shared<'a> = ();
    type MainThread<'a> = ();
}

impl DefaultPluginFactory for MyPlugin {
    fn get_descriptor() -> PluginDescriptor {
        PluginDescriptor::new("test", "test")
    }

    fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
        Ok(())
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        _shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(())
    }
}

const ENTRY: EntryDescriptor = clack_entry!(SinglePluginEntry<MyPlugin>);

#[test]
fn works_without_init() {
    // SAFETY: The given pointer is valid
    let result = unsafe { ENTRY.get_factory.unwrap()(CLAP_PLUGIN_FACTORY_ID.as_ptr()) };
    assert_ne!(result, core::ptr::null());
}
