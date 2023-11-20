#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]
#![deny(unsafe_code)]

use clack_extensions::audio_ports::{
    AudioPortFlags, AudioPortInfoData, AudioPortInfoWriter, AudioPortType, PluginAudioPorts,
    PluginAudioPortsImpl,
};

use crate::poly_oscillator::PolyOscillator;
use clack_plugin::prelude::*;

mod oscillator;
mod poly_oscillator;

pub struct PolySynthPlugin;

impl Plugin for PolySynthPlugin {
    type AudioProcessor<'a> = PolySynthAudioProcessor;
    type Shared<'a> = PolySynthPluginShared;
    type MainThread<'a> = PolySynthPluginMainThread;

    fn get_descriptor() -> Box<dyn PluginDescriptor> {
        use clack_plugin::plugin::descriptor::features::*;
        use std::ffi::CStr;

        Box::new(StaticPluginDescriptor {
            id: CStr::from_bytes_with_nul(b"org.rust-audio.clack.polysynth\0").unwrap(),
            name: CStr::from_bytes_with_nul(b"Clack PolySynth Example\0").unwrap(),
            features: Some(&[SYNTHESIZER, MONO, INSTRUMENT]),
            ..Default::default()
        })
    }

    fn declare_extensions(builder: &mut PluginExtensions<Self>, _shared: &PolySynthPluginShared) {
        builder.register::<PluginAudioPorts>();
    }
}

pub struct PolySynthAudioProcessor {
    poly_osc: PolyOscillator,
}

impl<'a> PluginAudioProcessor<'a, PolySynthPluginShared, PolySynthPluginMainThread>
    for PolySynthAudioProcessor
{
    fn activate(
        _host: HostAudioThreadHandle<'a>,
        _main_thread: &mut PolySynthPluginMainThread,
        _shared: &'a PolySynthPluginShared,
        audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            poly_osc: PolyOscillator::new(16, audio_config.sample_rate as f32),
        })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        let mut output_port = audio
            .output_port(0)
            .ok_or(PluginError::Message("No output"))?;

        let mut output_channels = output_port
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 output"))?;

        let output_buffer = output_channels
            .channel_mut(0)
            .ok_or(PluginError::Message("Expected at least one channel"))?;

        for event_batch in events.input.batch() {
            for event in event_batch.events() {
                self.poly_osc.process_event(event)
            }

            let output_buffer = &mut output_buffer[event_batch.sample_bounds()];
            self.poly_osc.generate_next_samples(output_buffer)
        }

        // If somehow the host didn't give us a mono output, we copy the output to all channels
        if output_channels.channel_count() > 1 {
            let (first_channel, other_channels) = output_channels.split_at_mut(1);
            let first_channel = first_channel.channel(0).unwrap();

            for other_channel in other_channels {
                other_channel.copy_from_slice(first_channel)
            }
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }
}

impl PluginAudioPortsImpl for PolySynthPluginMainThread {
    fn count(&self, is_input: bool) -> u32 {
        if is_input {
            0
        } else {
            1
        }
    }

    fn get(&self, is_input: bool, index: u32, writer: &mut AudioPortInfoWriter) {
        if is_input && index == 0 {
            writer.set(&AudioPortInfoData {
                id: 0,
                name: b"main",
                channel_count: 1,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::MONO),
                in_place_pair: None,
            });
        }
    }
}

pub struct PolySynthPluginShared;

impl<'a> PluginShared<'a> for PolySynthPluginShared {
    fn new(_host: HostHandle<'a>) -> Result<Self, PluginError> {
        Ok(Self)
    }
}

pub struct PolySynthPluginMainThread;

impl<'a> PluginMainThread<'a, PolySynthPluginShared> for PolySynthPluginMainThread {
    fn new(
        _host: HostMainThreadHandle<'a>,
        _shared: &'a PolySynthPluginShared,
    ) -> Result<Self, PluginError> {
        Ok(Self)
    }
}

clack_export_entry!(SinglePluginEntry<PolySynthPlugin>);
