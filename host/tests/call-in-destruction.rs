use clack_common::stream::{InputStream, OutputStream};
use clack_extensions::state::{PluginState, PluginStateImpl};
use clack_host::prelude::*;
use clack_plugin::clack_entry;
use clack_plugin::prelude::*;
use std::ffi::CStr;
use std::io::Write;
use std::sync::OnceLock;

struct MyPlugin;

impl Plugin for MyPlugin {
    type AudioProcessor<'a> = ();
    type Shared<'a> = ();
    type MainThread<'a> = MyPluginMainThread;

    fn declare_extensions(
        builder: &mut PluginExtensions<Self>,
        _shared: Option<&Self::Shared<'_>>,
    ) {
        builder.register::<PluginState>();
    }
}

struct MyPluginMainThread {
    data: String,
}

impl PluginMainThread<'_, ()> for MyPluginMainThread {}

impl PluginStateImpl for MyPluginMainThread {
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError> {
        output.write_all(self.data.as_bytes())?;
        Ok(())
    }

    fn load(&mut self, _input: &mut InputStream) -> Result<(), PluginError> {
        unimplemented!()
    }
}

impl DefaultPluginFactory for MyPlugin {
    fn get_descriptor() -> PluginDescriptor {
        PluginDescriptor::new("my.plugin", "My plugin")
    }

    fn new_shared(_host: HostSharedHandle) -> Result<Self::Shared<'_>, PluginError> {
        Ok(())
    }

    fn new_main_thread(
        _host: HostMainThreadHandle,
        _shared: &(),
    ) -> Result<MyPluginMainThread, PluginError> {
        Ok(MyPluginMainThread {
            data: "Hello world!".into(),
        })
    }
}

static MY_PLUGIN_ENTRY: EntryDescriptor = clack_entry!(SinglePluginEntry<MyPlugin>);

struct MyHost;

impl HostHandlers for MyHost {
    type Shared<'a> = MyHostShared<'a>;
    type MainThread<'a> = MyHostMainThread<'a>;
    type AudioProcessor<'a> = ();
}

struct MyHostShared<'a> {
    init: OnceLock<InitializingPluginHandle<'a>>,
}

impl<'a> SharedHandler<'a> for MyHostShared<'a> {
    fn initializing(&self, instance: InitializingPluginHandle<'a>) {
        let _ = self.init.set(instance);
    }

    fn request_restart(&self) {
        unimplemented!()
    }
    fn request_process(&self) {
        unimplemented!()
    }
    fn request_callback(&self) {
        unimplemented!()
    }
}

struct MyHostMainThread<'a> {
    instance: Option<InitializedPluginHandle<'a>>,
}

impl<'a> MainThreadHandler<'a> for MyHostMainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        assert!(instance.get_extension::<PluginState>().is_some());
        self.instance = Some(instance)
    }
}

impl Drop for MyHostMainThread<'_> {
    fn drop(&mut self) {
        let instance = self.instance.as_ref().unwrap();
        assert!(instance.get_extension::<PluginState>().is_none());
    }
}

#[test]
fn can_call_host_methods_during_init() {
    let host = HostInfo::new("host", "host", "host", "1.0").unwrap();

    let bundle = unsafe { PluginBundle::load_from_raw(&MY_PLUGIN_ENTRY, "/my/plugin") }.unwrap();
    let instance = PluginInstance::<MyHost>::new(
        |_| MyHostShared {
            init: OnceLock::new(),
        },
        |_| MyHostMainThread { instance: None },
        &bundle,
        CStr::from_bytes_with_nul(b"my.plugin\0").unwrap(),
        &host,
    )
    .unwrap();

    // This should try to read plugin data
    drop(instance)
}
