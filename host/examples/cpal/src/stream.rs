use crate::buffers::CpalAudioOutputBuffers;
use crate::host::CpalHost;
use clack_host::prelude::*;
use clack_host::process::StartedPluginAudioProcessor;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{
    BufferSize, BuildStreamError, Device, FromSample, OutputCallbackInfo, SampleFormat, SampleRate,
    Stream, StreamConfig,
};
use std::error::Error;

mod config;
mod midi;

use config::*;
use midi::*;

pub fn activate_to_stream(
    instance: &mut PluginInstance<CpalHost>,
) -> Result<Stream, Box<dyn Error>> {
    // Initialize CPAL
    let cpal_host = cpal::default_host();

    let output_device = cpal_host.default_output_device().unwrap();
    let default_config = output_device.default_output_config()?;

    let best = find_device_best_output_configs(&output_device)?;
    for config in best {
        println!("Supported output configuration: {config:?}");
    }

    let plugin_ports = find_config_from_ports(&instance.main_thread_plugin_data(), false);
    println!("Plugin port setup: {plugin_ports:?}");

    let midi = MidiReceiver::new(44_100)?;

    let config = StreamConfig {
        channels: 2,
        buffer_size: BufferSize::Fixed(1024),
        sample_rate: SampleRate(44_100),
    };

    let plugin_config = PluginAudioConfiguration {
        sample_rate: 44_100.0,
        frames_count_range: 1024..=1024,
    };

    let plugin_audio_processor = instance
        .activate(|_, _, _| (), plugin_config)?
        .start_processing()?;

    let audio_processor = StreamAudioProcessor::new(plugin_audio_processor, midi, 2, 1024);

    let stream = build_output_stream_for_sample_type(
        &output_device,
        audio_processor,
        &config,
        default_config.sample_format(),
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
        _ => todo!(),
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
        channel_count: usize,
        expected_buffer_size: usize,
    ) -> Self {
        Self {
            audio_processor: plugin_instance,
            buffers: CpalAudioOutputBuffers::with_capacity(channel_count, expected_buffer_size),
            midi_receiver,
            steady_counter: 0,
        }
    }

    pub fn process<S: FromSample<f32>>(&mut self, data: &mut [S]) {
        self.buffers.ensure_buffer_size_matches(data.len());

        let (ins, mut outs) = self.buffers.plugin_buffers();

        let sample_count = data.len() as u64;

        let events = if let Some(midi) = self.midi_receiver.as_mut() {
            midi.receive_all_events(sample_count)
        } else {
            InputEvents::empty()
        };

        match self.audio_processor.process(
            &ins,
            &mut outs,
            &events,
            &mut OutputEvents::void(),
            self.steady_counter,
            None,
            None,
        ) {
            Ok(_) => self.buffers.write_to(data),
            Err(e) => return eprintln!("{e}"),
        }

        self.steady_counter += data.len() as i64;
    }
}
