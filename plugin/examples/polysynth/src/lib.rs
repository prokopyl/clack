#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]
#![doc = include_str!("../README.md")]
#![deny(missing_docs, clippy::missing_docs_in_private_items, unsafe_code)]

use crate::params::PolySynthParams;
use crate::poly_oscillator::PolyOscillator;
use clack_extensions::state::PluginState;
use clack_extensions::{audio_ports::*, note_ports::*, params::*};
use clack_plugin::prelude::*;

mod oscillator;
mod params;
mod poly_oscillator;

/// The type that represents our plugin in Clack.
///
/// This is what implements the [`Plugin`] trait, and where all the other subtypes are attached.
pub struct PolySynthPlugin;

impl Plugin for PolySynthPlugin {
    type AudioProcessor<'a> = PolySynthAudioProcessor<'a>;
    type Shared<'a> = PolySynthPluginShared;
    type MainThread<'a> = PolySynthPluginMainThread<'a>;

    fn declare_extensions(builder: &mut PluginExtensions<Self>, _shared: &PolySynthPluginShared) {
        builder
            .register::<PluginAudioPorts>()
            .register::<PluginNotePorts>()
            .register::<PluginParams>()
            .register::<PluginState>();
    }
}

impl DefaultPluginFactory for PolySynthPlugin {
    fn get_descriptor() -> PluginDescriptor {
        use clack_plugin::plugin::features::*;

        PluginDescriptor::new("org.rust-audio.clack.polysynth", "Clack PolySynth Example")
            .with_features([SYNTHESIZER, MONO, INSTRUMENT])
    }

    fn new_shared(_host: HostHandle) -> Result<Self::Shared<'_>, PluginError> {
        Ok(PolySynthPluginShared {
            params: PolySynthParams::new(),
        })
    }

    fn new_main_thread<'a>(
        _host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(PolySynthPluginMainThread { shared })
    }
}

/// Our plugin's audio processor. It lives in the audio thread.
///
/// It receives note and parameter events, and generates a mono output by running the oscillators.
pub struct PolySynthAudioProcessor<'a> {
    /// The oscillator bank.
    poly_osc: PolyOscillator,
    /// A reference to the plugin's shared data.
    shared: &'a PolySynthPluginShared,
}

impl<'a> PluginAudioProcessor<'a, PolySynthPluginShared, PolySynthPluginMainThread<'a>>
    for PolySynthAudioProcessor<'a>
{
    fn activate(
        _host: HostAudioThreadHandle<'a>,
        _main_thread: &mut PolySynthPluginMainThread,
        shared: &'a PolySynthPluginShared,
        audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            poly_osc: PolyOscillator::new(16, audio_config.sample_rate as f32),
            shared,
        })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        // First, we have to make a few sanity checks.
        // We want at least a single output port, which contains at least one channel of `f32`
        // audio sample data.
        let mut output_port = audio
            .output_port(0)
            .ok_or(PluginError::Message("No output port found"))?;

        let mut output_channels = output_port
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 output"))?;

        let output_buffer = output_channels
            .channel_mut(0)
            .ok_or(PluginError::Message("Expected at least one channel"))?;

        // Ensure the buffer is zero-filled, as all oscillators will just add to it.
        output_buffer.fill(0.0);

        // We use the `EventBatcher` to handle incoming events in a sample-accurate way.
        for event_batch in events.input.batch() {
            // Handle all the events (note or param) for this batch.
            for event in event_batch.events() {
                self.poly_osc.handle_event(event);
                self.shared.params.handle_event(event);
            }

            // Received the updated volume parameter
            let volume = self.shared.params.get_volume();

            // With all the events out of the way, we can now handle a whole batch of sample
            // all at once.
            let output_buffer = &mut output_buffer[event_batch.sample_bounds()];
            self.poly_osc.generate_next_samples(output_buffer, volume);
        }

        // If somehow the host didn't give us a mono output, we copy the output to all channels
        if output_channels.channel_count() > 1 {
            let (first_channel, other_channels) = output_channels.split_at_mut(1);
            // PANIC: we just checked that channel_count is > 1.
            let first_channel = first_channel.channel(0).unwrap();

            // Copy the first channel into all the other channels.
            for other_channel in other_channels {
                other_channel.copy_from_slice(first_channel)
            }
        }

        // Return either the Continue state or the Sleep state, depending on if we have active
        // voices running or not.
        if self.poly_osc.has_active_voices() {
            Ok(ProcessStatus::Continue)
        } else {
            Ok(ProcessStatus::Sleep)
        }
    }

    fn stop_processing(&mut self) {
        // When audio processing stops, we stop all the oscillator voices just in case.
        self.poly_osc.stop_all();
    }
}

impl<'a> PluginAudioPortsImpl for PolySynthPluginMainThread<'a> {
    fn count(&mut self, is_input: bool) -> u32 {
        if is_input {
            0
        } else {
            1
        }
    }

    fn get(&mut self, is_input: bool, index: u32, writer: &mut AudioPortInfoWriter) {
        if !is_input && index == 0 {
            writer.set(&AudioPortInfoData {
                id: 1,
                name: b"main",
                channel_count: 1,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::MONO),
                in_place_pair: None,
            });
        }
    }
}

impl<'a> PluginNotePortsImpl for PolySynthPluginMainThread<'a> {
    fn count(&mut self, is_input: bool) -> u32 {
        if is_input {
            1
        } else {
            0
        }
    }

    fn get(&mut self, is_input: bool, index: u32, writer: &mut NotePortInfoWriter) {
        if is_input && index == 0 {
            writer.set(&NotePortInfoData {
                id: 1,
                name: b"main",
                preferred_dialect: Some(NoteDialect::Clap),
                supported_dialects: NoteDialects::CLAP,
            })
        }
    }
}

/// The plugin data that gets shared between the Main Thread and the Audio Thread.
pub struct PolySynthPluginShared {
    /// The plugin's parameter values.
    params: PolySynthParams,
}

impl<'a> PluginShared<'a> for PolySynthPluginShared {}

/// The data that belongs to the main thread of our plugin.
pub struct PolySynthPluginMainThread<'a> {
    /// A reference to the plugin's shared data.
    shared: &'a PolySynthPluginShared,
}

impl<'a> PluginMainThread<'a, PolySynthPluginShared> for PolySynthPluginMainThread<'a> {}

clack_export_entry!(SinglePluginEntry<PolySynthPlugin>);
