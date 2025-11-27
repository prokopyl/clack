use clack_extensions::log::{HostLog, HostLogImpl, LogSeverity};
use clack_host::factory::PluginFactory;
use clack_plugin::prelude::*;

use clack_host::prelude::*;
use clack_plugin::clack_entry;

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
        Err(PluginError::Message("Some error"))
    }
}

impl<'a> PluginAudioProcessor<'a, (), DivaPluginStubMainThread> for DivaPluginStubAudioProcessor {
    fn activate(
        _host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut DivaPluginStubMainThread,
        _shared: &'a (),
        _audio_config: PluginAudioConfiguration,
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
impl SharedHandler<'_> for MyHostShared {
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

impl HostLogImpl for MyHostShared {
    fn log(&self, severity: LogSeverity, message: &str) {
        // This is the error we're expecting
        if message != "Some error" {
            eprintln!("[{severity}] {message}");
        }
    }
}

struct MyHost;
impl HostHandlers for MyHost {
    type Shared<'a> = MyHostShared;

    type MainThread<'a> = ();
    type AudioProcessor<'a> = ();

    fn declare_extensions(builder: &mut HostExtensions<Self>, _: &Self::Shared<'_>) {
        builder.register::<HostLog>();
    }
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
        c"com.u-he.diva",
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
