use clack_host::factory::PluginFactory;
use clack_plugin::prelude::*;
use std::ffi::CStr;

use clack_host::prelude::*;
use clack_plugin::clack_entry;

pub struct DivaPluginStubAudioProcessor;
pub struct DivaPluginStub;
pub struct DivaPluginStubMainThread;

impl<'a> PluginMainThread<'a, ()> for DivaPluginStubMainThread {}

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

    fn new_shared(_host: HostSharedHandle) -> Result<Self::Shared<'_>, PluginError> {
        Ok(())
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        _shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Err(PluginError::AlreadyActivated)
    }
}

impl<'a> PluginAudioProcessor<'a, (), DivaPluginStubMainThread> for DivaPluginStubAudioProcessor {
    fn activate(
        _host: HostAudioThreadHandle<'a>,
        _main_thread: &mut DivaPluginStubMainThread,
        _shared: &'a (),
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        unreachable!()
    }

    fn process(
        &mut self,
        _process: Process,
        _audio: Audio,
        _events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        unreachable!()
    }
}

pub static DIVA_STUB_ENTRY: EntryDescriptor = clack_entry!(SinglePluginEntry<DivaPluginStub>);

struct MyHostShared;
impl<'a> HostShared<'a> for MyHostShared {
    fn request_restart(&self) {
        unreachable!()
    }
    fn request_process(&self) {
        unreachable!()
    }
    fn request_callback(&self) {
        unreachable!()
    }
}

struct MyHost;
impl Host for MyHost {
    type Shared<'a> = MyHostShared;

    type MainThread<'a> = ();
    type AudioProcessor<'a> = ();
}

#[test]
pub fn handles_instanciation_errors() {
    let bundle = unsafe {
        PluginBundle::load_from_raw(&DIVA_STUB_ENTRY, "/home/user/.clap/u-he/libdiva.so").unwrap()
    };
    let host_info =
        HostInfo::new("Legit Studio", "Legit Ltd.", "https://example.com", "4.3.2").unwrap();

    let plugin_instance = PluginInstance::<MyHost>::new(
        |_| MyHostShared,
        |_| (),
        &bundle,
        CStr::from_bytes_with_nul(b"com.u-he.diva\0").unwrap(),
        &host_info,
    );

    if plugin_instance.is_ok() {
        panic!("Instanciation should have failed")
    }
}

#[test]
pub fn it_works_concurrently_with_static_entrypoint() {
    let entrypoint = &DIVA_STUB_ENTRY;

    std::thread::scope(|s| {
        for i in 0..50 {
            std::thread::Builder::new()
                .name(format!("Test {i}"))
                .spawn_scoped(s, move || {
                    let bundle = unsafe {
                        PluginBundle::load_from_raw(entrypoint, "/home/user/.clap/u-he/libdiva.so")
                    }
                    .unwrap();

                    let desc = bundle
                        .get_factory::<PluginFactory>()
                        .unwrap()
                        .plugin_descriptor(0)
                        .unwrap();

                    assert_eq!(desc.id().unwrap().to_str().unwrap(), "com.u-he.diva");
                })
                .unwrap();
        }
    })
}
