use clack_plugin::plugin::descriptor::{PluginDescriptor, StaticPluginDescriptor};
use clack_plugin::prelude::*;
use std::ffi::CStr;

pub struct DivaPluginStub<'a> {
    shared: &'a DivaPluginStubShared<'a>,
}
pub struct DivaPluginStubShared<'a> {
    host: HostHandle<'a>,
}

impl<'a> PluginShared<'a> for DivaPluginStubShared<'a> {
    fn new(host: HostHandle<'a>) -> Result<Self, PluginError> {
        Ok(Self { host })
    }
}

impl<'a> Plugin<'a> for DivaPluginStub<'a> {
    type Shared = DivaPluginStubShared<'a>;
    type MainThread = ();

    fn get_descriptor() -> Box<dyn PluginDescriptor> {
        use clack_plugin::plugin::descriptor::features::*;

        Box::new(StaticPluginDescriptor {
            id: CStr::from_bytes_with_nul(b"com.u-he.diva\0").unwrap(),
            name: CStr::from_bytes_with_nul(b"Diva\0").unwrap(),
            features: Some(&[SYNTHESIZER, STEREO]),
            ..Default::default()
        })
    }

    fn activate(
        _host: HostAudioThreadHandle<'a>,
        _main_thread: &mut Self::MainThread,
        shared: &'a Self::Shared,
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self { shared })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        _events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        self.shared.host.request_callback();

        #[cfg(not(miri))]
        for event in _events.input {
            _events.output.try_push(event).unwrap();
        }

        let mut output_channels = audio.output_port(0).unwrap().channels().unwrap();
        let output_buf = output_channels.as_f32_mut().unwrap().iter_mut();

        for channel in output_buf {
            for (input, output) in [42.0f32, 69.0, 21.0, 34.5].iter().zip(channel.iter_mut()) {
                *output = *input;
            }
        }
        Ok(ProcessStatus::Sleep)
    }
}

#[allow(unused)] // This is only used in doctests
pub static DIVA_STUB_ENTRY: PluginEntryDescriptor = SinglePluginEntry::<DivaPluginStub>::DESCRIPTOR;
