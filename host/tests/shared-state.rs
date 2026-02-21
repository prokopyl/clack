use clack_host::prelude::*;
use clack_plugin::prelude::*;
use std::sync::OnceLock;

pub struct DivaPluginStubAudioProcessor;
pub struct DivaPluginStub;
pub struct DivaPluginStubMainThread;

impl PluginMainThread<'_, ()> for DivaPluginStubMainThread {}

impl Plugin for DivaPluginStub {
    type AudioProcessor<'a> = DivaPluginStubAudioProcessor;
    type Shared<'a> = ();
    type MainThread<'a> = DivaPluginStubMainThread;
}

impl DefaultPluginFactory for DivaPluginStub {
    fn get_descriptor() -> PluginDescriptor {
        use clack_plugin::plugin::features::*;

        PluginDescriptor::new("com.u-he.diva", "Diva").with_features([SYNTHESIZER, STEREO])
    }

    fn new_shared(_host: HostSharedHandle<'_>) -> Result<Self::Shared<'_>, PluginError> {
        Ok(())
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        _shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(DivaPluginStubMainThread {})
    }
}

impl<'a> PluginAudioProcessor<'a, (), DivaPluginStubMainThread> for DivaPluginStubAudioProcessor {
    fn activate(
        _host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut DivaPluginStubMainThread,
        _shared: &'a (),
        _audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        unimplemented!()
    }

    fn process(
        &mut self,
        _process: Process,
        _audio: Audio,
        _events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        unimplemented!()
    }
}

struct MyHostShared {
    state_ext: OnceLock<bool>,
}

impl<'a> SharedHandler<'a> for MyHostShared {
    fn initializing(&self, _instance: InitializingPluginHandle<'a>) {
        match self.state_ext.set(true) {
            Ok(_) => {}
            Err(_) => panic!("Failed to set state ext"),
        };
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

struct MyHost;
impl HostHandlers for MyHost {
    type Shared<'a> = MyHostShared;

    type MainThread<'a> = ();
    type AudioProcessor<'a> = ();
}

#[test]
pub fn handles_drop_order() {
    let entry = PluginEntry::load_from_clack::<SinglePluginEntry<DivaPluginStub>>(
        c"/home/user/.clap/u-he/libdiva.so",
    )
    .unwrap();
    let host_info =
        HostInfo::new("Legit Studio", "Legit Ltd.", "https://example.com", "4.3.2").unwrap();

    let plugin_instance = PluginInstance::<MyHost>::new(
        |_| MyHostShared {
            state_ext: OnceLock::new(),
        },
        |_| (),
        &entry,
        c"com.u-he.diva",
        &host_info,
    )
    .unwrap();

    let _ext = plugin_instance.access_shared_handler(|s| s.state_ext.get());
}
