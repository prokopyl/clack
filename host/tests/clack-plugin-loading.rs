use clack_plugin::plugin::descriptor::StaticPluginDescriptor;
use clack_plugin::prelude::*;
use clack_plugin::process::audio::ChannelPair;
use clack_test_host::TestHost;
use std::ffi::CStr;

pub struct GainPlugin;

impl Plugin for GainPlugin {
    type AudioProcessor<'a> = GainPluginAudioProcessor<'a>;
    type Shared<'a> = GainPluginShared<'a>;
    type MainThread<'a> = GainPluginMainThread<'a>;

    fn get_descriptor() -> Box<dyn PluginDescriptor> {
        use clack_plugin::plugin::descriptor::features::*;

        Box::new(StaticPluginDescriptor {
            id: CStr::from_bytes_with_nul(b"org.rust-audio.clack.gain\0").unwrap(),
            name: CStr::from_bytes_with_nul(b"Clack Gain Example\0").unwrap(),
            features: Some(&[SYNTHESIZER, STEREO]),
            ..Default::default()
        })
    }
}

pub struct GainPluginAudioProcessor<'a> {
    _host: HostAudioThreadHandle<'a>,
}

impl<'a> PluginAudioProcessor<'a, GainPluginShared<'a>, GainPluginMainThread<'a>>
    for GainPluginAudioProcessor<'a>
{
    fn activate(
        host: HostAudioThreadHandle<'a>,
        _main_thread: &mut GainPluginMainThread,
        _shared: &'a GainPluginShared,
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self { _host: host })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        _events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        for channel_pair in audio
            .port_pairs()
            // Filter out any non-f32 data, in case host is misbehaving and sends f64 data
            .filter_map(|mut p| p.channels().ok()?.into_f32())
            .flatten()
        {
            let buf = match channel_pair {
                ChannelPair::InputOnly(_) => continue, // Ignore extra inputs
                ChannelPair::OutputOnly(o) => {
                    // Just set extra outputs to 0
                    o.fill(0.0);
                    continue;
                }
                ChannelPair::InputOutput(i, o) => {
                    o.copy_from_slice(i);
                    o
                }
                ChannelPair::InPlace(o) => o,
            };

            for x in buf {
                *x *= 2.0;
            }
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }
}

pub struct GainPluginShared<'a> {
    _host: HostHandle<'a>,
}

impl<'a> PluginShared<'a> for GainPluginShared<'a> {
    fn new(host: HostHandle<'a>) -> Result<Self, PluginError> {
        Ok(Self { _host: host })
    }
}

pub struct GainPluginMainThread<'a> {
    _host: HostMainThreadHandle<'a>,
}

impl<'a> PluginMainThread<'a, GainPluginShared<'a>> for GainPluginMainThread<'a> {
    fn new(
        host: HostMainThreadHandle<'a>,
        _shared: &'a GainPluginShared,
    ) -> Result<Self, PluginError> {
        Ok(Self { _host: host })
    }
}

clack_export_entry!(SinglePluginEntry<GainPlugin>);

#[test]
fn it_works() {
    // Initialize host
    let mut host = TestHost::instantiate(&clap_entry);

    host.activate();

    host.inputs_mut()[0].fill(69f32);
    host.inputs_mut()[1].fill(69f32);

    host.process().unwrap();

    // Check the gain was applied properly (x2)
    for channel_index in 0..1 {
        let inbuf = &host.inputs()[channel_index];
        let outbuf = &host.outputs()[channel_index];
        for (input, output) in inbuf.iter().zip(outbuf.iter()) {
            assert_eq!(*output, *input * 2.0)
        }
    }

    host.deactivate();
}
