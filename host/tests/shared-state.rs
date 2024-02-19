use clack_host::prelude::*;
use clack_plugin::clack_entry;
use clack_plugin::prelude::*;
use std::ffi::CStr;
use std::sync::OnceLock;

pub struct DivaPluginStubAudioProcessor;
pub struct DivaPluginStub;
pub struct DivaPluginStubMainThread;

impl<'a> PluginMainThread<'a, ()> for DivaPluginStubMainThread {
    fn new(_host: HostMainThreadHandle<'a>, _shared: &'a ()) -> Result<Self, PluginError> {
        Ok(Self)
    }
}

impl Plugin for DivaPluginStub {
    type AudioProcessor<'a> = DivaPluginStubAudioProcessor;
    type Shared<'a> = ();
    type MainThread<'a> = DivaPluginStubMainThread;

    fn get_descriptor() -> PluginDescriptor {
        use clack_plugin::plugin::features::*;

        PluginDescriptor::new("com.u-he.diva", "Diva").with_features([SYNTHESIZER, STEREO])
    }
}

impl<'a> PluginAudioProcessor<'a, (), DivaPluginStubMainThread> for DivaPluginStubAudioProcessor {
    fn activate(
        _host: HostAudioThreadHandle<'a>,
        _main_thread: &mut DivaPluginStubMainThread,
        _shared: &'a (),
        _audio_config: AudioConfiguration,
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

pub static DIVA_STUB_ENTRY: EntryDescriptor = clack_entry!(SinglePluginEntry<DivaPluginStub>);

struct MyHostShared {
    state_ext: OnceLock<bool>,
}

impl<'a> HostShared<'a> for MyHostShared {
    fn instantiated(&self, _instance: PluginSharedHandle<'a>) {
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

struct MyHostMainThread<'a> {
    shared: &'a MyHostShared,
}

impl<'a> HostMainThread<'a> for MyHostMainThread<'a> {}

struct MyHost;
impl Host for MyHost {
    type Shared<'a> = MyHostShared;

    type MainThread<'a> = MyHostMainThread<'a>;
    type AudioProcessor<'a> = ();
}

#[test]
pub fn handles_drop_order() {
    let bundle = unsafe {
        PluginBundle::load_from_raw(&DIVA_STUB_ENTRY, "/home/user/.clap/u-he/libdiva.so").unwrap()
    };
    let host_info =
        HostInfo::new("Legit Studio", "Legit Ltd.", "https://example.com", "4.3.2").unwrap();

    let plugin_instance = PluginInstance::<MyHost>::new(
        |_| MyHostShared {
            state_ext: OnceLock::new(),
        },
        |shared| MyHostMainThread { shared },
        &bundle,
        CStr::from_bytes_with_nul(b"com.u-he.diva\0").unwrap(),
        &host_info,
    )
    .unwrap();

    let _ext = plugin_instance
        .main_thread_host_data()
        .shared
        .state_ext
        .get();
}
