use crate::host::CpalHost;
use clack_host::prelude::*;
use clack_host::process::StartedPluginAudioProcessor;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{
    BuildStreamError, Device, FromSample, OutputCallbackInfo, SampleFormat, Stream, StreamConfig,
};
use std::error::Error;
use std::time::Instant;

mod buffers;
mod config;
mod midi;

use buffers::*;
use config::*;
use midi::*;

pub fn activate_to_stream(
    instance: &mut PluginInstance<CpalHost>,
) -> Result<Stream, Box<dyn Error>> {
    // Initialize CPAL
    let cpal_host = cpal::default_host();

    let output_device = cpal_host.default_output_device().unwrap();

    let config = FullAudioConfig::negociate_from(&output_device, instance)?;
    println!("Using negociated audio output settings: {config}");

    let midi = MidiReceiver::new(44_100)?;

    let plugin_audio_processor = instance
        .activate(|_, _, _| (), config.as_clack_plugin_config())?
        .start_processing()?;

    let sample_format = config.sample_format;
    let cpal_config = config.as_cpal_stream_config();
    let audio_processor = StreamAudioProcessor::new(plugin_audio_processor, midi, config);

    let stream = build_output_stream_for_sample_type(
        &output_device,
        audio_processor,
        &cpal_config,
        sample_format,
    )?;

    Ok(stream)
}

fn build_output_stream_for_sample_type(
    device: &Device,
    processor: StreamAudioProcessor,
    config: &StreamConfig,
    sample_type: SampleFormat,
) -> Result<Stream, BuildStreamError> {
    let err = |e| eprintln!("{e}");

    match sample_type {
        SampleFormat::I8 => {
            device.build_output_stream(config, make_stream_runner::<i8>(processor), err, None)
        }
        SampleFormat::I16 => {
            device.build_output_stream(config, make_stream_runner::<i16>(processor), err, None)
        }
        SampleFormat::I32 => {
            device.build_output_stream(config, make_stream_runner::<i32>(processor), err, None)
        }
        SampleFormat::I64 => {
            device.build_output_stream(config, make_stream_runner::<i64>(processor), err, None)
        }
        SampleFormat::U8 => {
            device.build_output_stream(config, make_stream_runner::<u8>(processor), err, None)
        }
        SampleFormat::U16 => {
            device.build_output_stream(config, make_stream_runner::<u16>(processor), err, None)
        }
        SampleFormat::U32 => {
            device.build_output_stream(config, make_stream_runner::<u32>(processor), err, None)
        }
        SampleFormat::U64 => {
            device.build_output_stream(config, make_stream_runner::<u64>(processor), err, None)
        }
        SampleFormat::F32 => {
            device.build_output_stream(config, make_stream_runner::<f32>(processor), err, None)
        }
        SampleFormat::F64 => {
            device.build_output_stream(config, make_stream_runner::<f64>(processor), err, None)
        }
        f => unimplemented!("Unknown sample format: {f:?}"),
    }
}

fn make_stream_runner<S: FromSample<f32>>(
    mut audio_processor: StreamAudioProcessor,
) -> impl FnMut(&mut [S], &OutputCallbackInfo) {
    move |data, _info| audio_processor.process(data)
}

struct StreamAudioProcessor {
    audio_processor: StartedPluginAudioProcessor<CpalHost>,
    buffers: CpalAudioOutputBuffers,
    midi_receiver: Option<MidiReceiver>,
    steady_counter: i64,
}

impl StreamAudioProcessor {
    pub fn new(
        plugin_instance: StartedPluginAudioProcessor<CpalHost>,
        midi_receiver: Option<MidiReceiver>,
        config: FullAudioConfig,
    ) -> Self {
        Self {
            audio_processor: plugin_instance,
            buffers: CpalAudioOutputBuffers::from_config(config),
            midi_receiver,
            steady_counter: 0,
        }
    }

    pub fn process<S: FromSample<f32>>(&mut self, data: &mut [S]) {
        self.buffers.ensure_buffer_size_matches(data.len());
        let sample_count = self.buffers.cpal_buf_len_to_sample_count(data.len());

        let (ins, mut outs) = self.buffers.plugin_buffers(data.len());

        let events = if let Some(midi) = self.midi_receiver.as_mut() {
            midi.receive_all_events(sample_count as u64)
        } else {
            InputEvents::empty()
        };

        match self.audio_processor.process(
            &ins,
            &mut outs,
            &events,
            &mut OutputEvents::void(),
            self.steady_counter,
            Some(sample_count),
            None,
        ) {
            Ok(_) => self.buffers.write_to(data),
            Err(e) => eprintln!("{e}"),
        }

        self.steady_counter += sample_count as i64;
    }
}
