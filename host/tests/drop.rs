use clack_plugin::prelude::*;
use std::ffi::CStr;
use std::thread;
use std::thread::{current, ThreadId};
use std::time::Duration;

use clack_host::prelude::*;
use clack_plugin::clack_entry;

pub struct DivaPluginStubAudioProcessor {
    processing: bool,
}

pub struct DivaPluginStub;
pub struct DivaPluginStubMainThread {
    thread_id: ThreadId,
    active: bool,
}

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

    fn new_shared(_host: HostHandle) -> Result<Self::Shared<'_>, PluginError> {
        Ok(())
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        _shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(DivaPluginStubMainThread {
            active: false,
            thread_id: current().id(),
        })
    }
}

impl<'a> PluginAudioProcessor<'a, (), DivaPluginStubMainThread> for DivaPluginStubAudioProcessor {
    fn activate(
        _host: HostAudioThreadHandle<'a>,
        main_thread: &mut DivaPluginStubMainThread,
        _shared: &'a (),
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        assert!(!main_thread.active);
        main_thread.active = true;

        Ok(Self { processing: false })
    }

    fn process(
        &mut self,
        _process: Process,
        _audio: Audio,
        _events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        assert!(self.processing);

        Ok(ProcessStatus::Sleep)
    }

    fn deactivate(self, main_thread: &mut DivaPluginStubMainThread) {
        assert!(!self.processing);
        assert!(main_thread.active);
        main_thread.active = false;
    }

    fn start_processing(&mut self) -> Result<(), PluginError> {
        assert!(!self.processing);
        self.processing = true;
        Ok(())
    }

    fn stop_processing(&mut self) {
        assert!(self.processing);
        self.processing = false;
    }
}

impl Drop for DivaPluginStubAudioProcessor {
    fn drop(&mut self) {
        assert!(!self.processing);
    }
}

impl Drop for DivaPluginStubMainThread {
    fn drop(&mut self) {
        assert!(!self.active);
        assert_eq!(self.thread_id, current().id())
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

fn instantiate() -> PluginInstance<MyHost> {
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

    plugin_instance.unwrap()
}

#[test]
pub fn handles_normal_deactivate() {
    let mut instance = instantiate();
    let config = PluginAudioConfiguration {
        sample_rate: 44_100.0,
        frames_count_range: 5..=5,
    };

    let processor = instance.activate(|_, _, _| (), config).unwrap();
    instance.deactivate(processor);
}

#[test]
pub fn handles_try_deactivate() {
    let mut instance = instantiate();
    let config = PluginAudioConfiguration {
        sample_rate: 44_100.0,
        frames_count_range: 5..=5,
    };

    let processor = instance.activate(|_, _, _| (), config).unwrap();

    assert!(instance.try_deactivate().is_err());
    drop(processor);

    instance.try_deactivate().unwrap();
}

#[test]
pub fn stops_when_dropping() {
    let mut instance = instantiate();
    let config = PluginAudioConfiguration {
        sample_rate: 44_100.0,
        frames_count_range: 5..=5,
    };

    let processor = instance.activate(|_, _, _| (), config).unwrap();
    let processor = processor.start_processing().unwrap();

    drop(processor);
    drop(instance);
}

#[test]
pub fn works_with_reverse_drop() {
    let mut instance = instantiate();
    let config = PluginAudioConfiguration {
        sample_rate: 44_100.0,
        frames_count_range: 5..=5,
    };

    let processor = instance.activate(|_, _, _| (), config).unwrap();
    let processor = processor.start_processing().unwrap();

    drop(instance);
    drop(processor);
}

#[test]
pub fn works_with_forgotten_audio_processor() {
    let mut instance = instantiate();
    let config = PluginAudioConfiguration {
        sample_rate: 44_100.0,
        frames_count_range: 5..=5,
    };

    let processor = instance.activate(|_, _, _| (), config).unwrap();

    let t = thread::spawn(move || {
        let processor = processor.start_processing().unwrap();
        thread::sleep(Duration::from_millis(200));
        drop(processor);
    });

    drop(instance);

    t.join().unwrap();
}
