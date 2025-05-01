#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]
#![deny(missing_docs, clippy::missing_docs_in_private_items, unsafe_code)]
#![doc = include_str!("../README.md")]

use crate::params::GainParams;
use clack_extensions::state::PluginState;
use clack_extensions::wrappers::auv2::{
    PluginFactoryAsAUV2, PluginFactoryAsAUV2Wrapper, PluginInfoAsAuv2,
};
use clack_extensions::wrappers::vst3::{
    PluginAsVST3, PluginAsVST3Impl, PluginFactoryAsVST3, PluginFactoryAsVST3Wrapper,
    PluginInfoAsVST3,
};
use clack_extensions::{audio_ports::*, params::*};
use clack_plugin::entry::{EntryFactories, EntryLoadError};
use clack_plugin::prelude::*;
use std::ffi::CStr;

mod params;

/// The type that represents our plugin in Clack.
///
/// This is what implements the [`Plugin`] trait, where all the other subtypes are attached.
pub struct GainPlugin;

impl Plugin for GainPlugin {
    type AudioProcessor<'a> = GainPluginAudioProcessor<'a>;
    type Shared<'a> = GainPluginShared;
    type MainThread<'a> = GainPluginMainThread<'a>;

    fn declare_extensions(
        builder: &mut PluginExtensions<Self>,
        _shared: Option<&GainPluginShared>,
    ) {
        builder
            .register::<PluginAudioPorts>()
            .register::<PluginParams>()
            .register::<PluginState>()
            .register::<PluginAsVST3>();
    }
}

impl DefaultPluginFactory for GainPlugin {
    fn get_descriptor() -> PluginDescriptor {
        use clack_plugin::plugin::features::*;

        PluginDescriptor::new("org.rust-audio.clack.gain", "Clack Gain Example")
            .with_features([AUDIO_EFFECT, STEREO])
    }

    fn new_shared(_host: HostSharedHandle) -> Result<Self::Shared<'_>, PluginError> {
        Ok(GainPluginShared {
            params: GainParams::new(),
        })
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(Self::MainThread { shared })
    }
}

/// Our plugin's audio processor. It lives in the audio thread.
///
/// It receives parameter events, and process a stereo audio signal by operating on the given audio
/// buffer.
pub struct GainPluginAudioProcessor<'a> {
    /// A reference to the plugin's shared data.
    shared: &'a GainPluginShared,
}

impl<'a> PluginAudioProcessor<'a, GainPluginShared, GainPluginMainThread<'a>>
    for GainPluginAudioProcessor<'a>
{
    fn activate(
        _host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut GainPluginMainThread,
        shared: &'a GainPluginShared,
        _audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        // This is where we would allocate intermediate buffers and such if we needed them.
        Ok(Self { shared })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        // First, we have to make a few sanity checks.
        // We want at least a single input/output port pair, which contains channels of `f32`
        // audio sample data.
        let mut port_pair = audio
            .port_pair(0)
            .ok_or(PluginError::Message("No input/output ports found"))?;

        let mut output_channels = port_pair
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        let mut channel_buffers = [None, None];

        // Extract the buffer slices that we need, while making sure they are paired correctly and
        // check for either in-place or separate buffers.
        for (pair, buf) in output_channels.iter_mut().zip(&mut channel_buffers) {
            *buf = match pair {
                ChannelPair::InputOnly(_) => None,
                ChannelPair::OutputOnly(_) => None,
                ChannelPair::InPlace(b) => Some(b),
                ChannelPair::InputOutput(i, o) => {
                    o.copy_from_slice(i);
                    Some(o)
                }
            }
        }

        // Now let's process the audio, while splitting the processing in batches between each
        // sample-accurate event.

        for event_batch in events.input.batch() {
            // Process all param events in this batch
            for event in event_batch.events() {
                self.shared.params.handle_event(event)
            }

            // Get the volume value after all parameter changes have been handled.
            let volume = self.shared.params.get_volume();

            for buf in channel_buffers.iter_mut().flatten() {
                for sample in buf.iter_mut() {
                    *sample *= volume
                }
            }
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }
}

impl PluginAudioPortsImpl for GainPluginMainThread<'_> {
    fn count(&mut self, _is_input: bool) -> u32 {
        1
    }

    fn get(&mut self, index: u32, _is_input: bool, writer: &mut AudioPortInfoWriter) {
        if index == 0 {
            writer.set(&AudioPortInfo {
                id: ClapId::new(0),
                name: b"main",
                channel_count: 2,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::STEREO),
                in_place_pair: None,
            });
        }
    }
}

/// The plugin data that gets shared between the Main Thread and the Audio Thread.
pub struct GainPluginShared {
    /// The plugin's parameter values.
    params: GainParams,
}

impl PluginShared<'_> for GainPluginShared {}

impl PluginAsVST3Impl for GainPluginShared {
    fn num_midi_channels(&self, note_port: u32) -> u32 {
        42
    }

    fn supported_note_expressions(&self) -> u32 {
        0
    }
}

/// The data that belongs to the main thread of our plugin.
pub struct GainPluginMainThread<'a> {
    /// A reference to the plugin's shared data.
    shared: &'a GainPluginShared,
}

impl<'a> PluginMainThread<'a, GainPluginShared> for GainPluginMainThread<'a> {}

/// f
struct MyAuV2Factory;

impl PluginFactoryAsAUV2 for MyAuV2Factory {
    fn get_auv2_info(&self, index: u32) -> Option<PluginInfoAsAuv2> {
        Some(PluginInfoAsAuv2::new("no", "idea"))
    }
}

/// f
struct MyVST3Factory;

impl PluginFactoryAsVST3 for MyVST3Factory {
    fn get_vst3_info(&self, index: u32) -> Option<&PluginInfoAsVST3> {
        static INFO: PluginInfoAsVST3<'static> =
            PluginInfoAsVST3::new(Some(c"foo"), Some(b"birbbirbbirbbirb"), Some(c"baz"));

        Some(&INFO)
    }
}

/// F
struct MyEntry {
    /// F
    auv2_factory: PluginFactoryAsAUV2Wrapper<MyAuV2Factory>,
}
impl Entry for MyEntry {
    fn new(bundle_path: &CStr) -> Result<Self, EntryLoadError> {
        Ok(Self {
            auv2_factory: PluginFactoryAsAUV2Wrapper::new(
                c"birb_made",
                c"Birb Made ™",
                MyAuV2Factory,
            ),
        })
    }

    fn declare_factories<'a>(&'a self, builder: &mut EntryFactories<'a>) {
        static WRAPPER: PluginFactoryAsVST3Wrapper<MyVST3Factory> =
            PluginFactoryAsVST3Wrapper::new(None, None, None, MyVST3Factory);

        builder
            .register_factory(&self.auv2_factory)
            .register_factory(&WRAPPER);
    }
}

clack_export_entry!(SinglePluginEntry<GainPlugin>);
