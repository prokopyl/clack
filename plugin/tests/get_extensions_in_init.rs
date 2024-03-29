use clack_extensions::timer::*;
use clack_host::prelude::*;
use clack_plugin::prelude::*;

struct MyPlugin;
struct MyPluginMainThread;

impl Plugin for MyPlugin {
    type AudioProcessor<'a> = ();
    type Shared<'a> = ();
    type MainThread<'a> = MyPluginMainThread;

    fn declare_extensions(builder: &mut PluginExtensions<Self>, shared: &Self::Shared<'_>) {
        builder.register::<PluginTimer>();
    }
}

impl<'a> PluginMainThread<'a, ()> for MyPluginMainThread {}

impl PluginTimerImpl for MyPluginMainThread {
    fn on_timer(&mut self, timer_id: TimerId) {
        assert_eq!(timer_id.0, 42);
    }
}

struct MyHost;
struct MyHostMainThread;

impl Host for MyHost {
    type Shared<'a> = ();
    type MainThread<'a> = MyHostMainThread;
    type AudioProcessor<'a> = ();

    fn declare_extensions(builder: &mut HostExtensions<Self>, _shared: &Self::Shared<'_>) {
        builder.register::<HostTimer>();
    }
}

impl<'a> HostMainThread<'a> for MyHostMainThread {
    fn instantiated(&mut self, instance: PluginMainThreadHandle<'a>) {
        todo!()
    }
}

impl HostTimerImpl for MyHostMainThread {
    fn register_timer(&mut self, period_ms: u32) -> Result<TimerId, TimerError> {
        todo!()
    }

    fn unregister_timer(&mut self, _timer_id: TimerId) -> Result<(), TimerError> {
        unreachable!()
    }
}

#[test]
pub fn works() {}
