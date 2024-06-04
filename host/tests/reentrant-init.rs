use clack_extensions::timer::{HostTimer, HostTimerImpl, PluginTimer, PluginTimerImpl, TimerId};
use clack_host::prelude::*;
use clack_plugin::clack_entry;
use clack_plugin::prelude::*;
use std::ffi::CStr;
use std::sync::OnceLock;

struct MyPlugin;

impl Plugin for MyPlugin {
    type AudioProcessor<'a> = ();
    type Shared<'a> = ();
    type MainThread<'a> = MyPluginMainThread;

    fn declare_extensions(builder: &mut PluginExtensions<Self>, shared: Option<&Self::Shared<'_>>) {
        assert!(shared.is_none()); // Host will only query extensions from within.
        builder.register::<PluginTimer>();
    }
}

struct MyPluginMainThread;

impl<'a> PluginMainThread<'a, ()> for MyPluginMainThread {}

impl PluginTimerImpl for MyPluginMainThread {
    fn on_timer(&mut self, timer_id: TimerId) {
        assert_eq!(timer_id, TimerId(5));
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
        mut host: HostMainThreadHandle,
        _shared: &(),
    ) -> Result<MyPluginMainThread, PluginError> {
        let timer: HostTimer = host.get_extension().unwrap();
        let timer_id = timer.register_timer(&mut host, 1_000).unwrap();
        assert_eq!(timer_id, TimerId(5));
        Ok(MyPluginMainThread)
    }
}

static MY_PLUGIN_ENTRY: EntryDescriptor = clack_entry!(SinglePluginEntry<MyPlugin>);

struct MyHost;

impl HostHandlers for MyHost {
    type Shared<'a> = MyHostShared<'a>;
    type MainThread<'a> = MyHostMainThread<'a>;
    type AudioProcessor<'a> = ();

    fn declare_extensions(builder: &mut HostExtensions<Self>, _shared: &Self::Shared<'_>) {
        builder.register::<HostTimer>();
    }
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
    shared: &'a MyHostShared<'a>,
    timer_registered: bool,
}

impl<'a> MainThreadHandler<'a> for MyHostMainThread<'a> {
    fn initialized(&mut self, _instance: InitializedPluginHandle<'a>) {}
}

impl<'a> HostTimerImpl for MyHostMainThread<'a> {
    fn register_timer(&mut self, period_ms: u32) -> Result<TimerId, HostError> {
        assert_eq!(period_ms, 1000);

        let handle = self
            .shared
            .init
            .get()
            .expect("Initializing should have been called already!");

        handle
            .get_extension::<PluginTimer>()
            .expect("Plugin should implement Timer extension!");

        self.timer_registered = true;
        Ok(TimerId(5))
    }

    fn unregister_timer(&mut self, _timer_id: TimerId) -> Result<(), HostError> {
        unimplemented!()
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
        |shared| MyHostMainThread {
            shared,
            timer_registered: false,
        },
        &bundle,
        CStr::from_bytes_with_nul(b"my.plugin\0").unwrap(),
        &host,
    )
    .unwrap();

    // Timer should have already been registered by the plugin during init().
    assert!(instance.access_handler(|h| h.timer_registered));
}
