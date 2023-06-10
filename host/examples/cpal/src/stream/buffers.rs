use crate::stream::config::FullAudioConfig;
use clack_host::prelude::{
    AudioPortBuffer, AudioPortBufferType, AudioPorts, InputAudioBuffers, InputChannel,
    OutputAudioBuffers,
};
use cpal::{FromSample, Sample};

pub struct CpalAudioOutputBuffers {
    config: FullAudioConfig,

    input_ports: AudioPorts,
    output_ports: AudioPorts,

    input_port_channels: Box<[Vec<f32>]>,
    output_port_channels: Box<[Vec<f32>]>,

    muxed: Vec<f32>,
    actual_frame_count: usize,
}

impl CpalAudioOutputBuffers {
    pub fn from_config(config: FullAudioConfig) -> Self {
        if config.output_channel_count > 2 {
            panic!(
                "Unsupported {}-channel layout: this example only supports mono or stereo output.",
                config.output_channel_count
            )
        }

        let total_input_channel_count = config.plugin_input_port_config.total_channel_count();
        let total_output_channel_count = config.plugin_output_port_config.total_channel_count();
        let frame_count = config.max_buffer_size as usize;

        Self {
            input_ports: AudioPorts::with_capacity(
                total_input_channel_count,
                config.plugin_input_port_config.ports.len(),
            ),
            output_ports: AudioPorts::with_capacity(
                total_output_channel_count,
                config.plugin_output_port_config.ports.len(),
            ),
            input_port_channels: config
                .plugin_input_port_config
                .ports
                .iter()
                .map(|p| vec![0.0; frame_count * p.port_layout.channel_count() as usize])
                .collect(),
            output_port_channels: config
                .plugin_output_port_config
                .ports
                .iter()
                .map(|p| vec![0.0; frame_count * p.port_layout.channel_count() as usize])
                .collect(),
            muxed: vec![0.0; frame_count * config.output_channel_count],
            config,
            actual_frame_count: frame_count,
        }
    }

    pub fn ensure_buffer_size_matches(&mut self, total_buffer_size: usize) {
        // println!("{}", total_buffer_size);
        let current_frame_count = self.cpal_buf_len_to_sample_count(total_buffer_size);

        if self.actual_frame_count < current_frame_count {
            println!("Warn: Expected buffer of length {} at most, but CPAL provided buffer of length {}. Reallocating.", self.actual_frame_count, current_frame_count);
            self.actual_frame_count = current_frame_count;

            for (buf, port) in self
                .input_port_channels
                .iter_mut()
                .zip(&self.config.plugin_input_port_config.ports)
            {
                buf.resize(
                    current_frame_count * port.port_layout.channel_count() as usize,
                    0.0,
                );
            }

            for (buf, port) in self
                .output_port_channels
                .iter_mut()
                .zip(&self.config.plugin_output_port_config.ports)
            {
                buf.resize(
                    current_frame_count * port.port_layout.channel_count() as usize,
                    0.0,
                );
            }

            self.muxed
                .resize(current_frame_count * self.config.output_channel_count, 0.0);
        }
    }

    pub fn cpal_buf_len_to_sample_count(&self, buf_len: usize) -> usize {
        buf_len / self.config.output_channel_count
    }

    pub fn plugin_buffers(
        &mut self,
        cpal_buf_len: usize,
    ) -> (InputAudioBuffers, OutputAudioBuffers) {
        let sample_count = self.cpal_buf_len_to_sample_count(cpal_buf_len);
        assert!(sample_count <= self.actual_frame_count);

        // just in case
        self.output_port_channels
            .iter_mut()
            .for_each(|b| b.fill(0.0));
        self.input_port_channels
            .iter_mut()
            .for_each(|b| b.fill(0.0));

        (
            self.input_ports
                .with_input_buffers(self.input_port_channels.iter_mut().map(|port_buf| {
                    AudioPortBuffer {
                        latency: 0,
                        channels: AudioPortBufferType::f32_input_only(
                            port_buf
                                .chunks_exact_mut(self.actual_frame_count)
                                .map(|buffer| InputChannel {
                                    buffer: &mut buffer[..sample_count],
                                    is_constant: true,
                                }),
                        ),
                    }
                })),
            self.output_ports
                .with_output_buffers(self.output_port_channels.iter_mut().map(|port_buf| {
                    AudioPortBuffer {
                        latency: 0,
                        channels: AudioPortBufferType::f32_output_only(
                            port_buf
                                .chunks_exact_mut(self.actual_frame_count)
                                .map(|buf| &mut buf[..sample_count]),
                        ),
                    }
                })),
        )
    }

    pub fn write_to<S: FromSample<f32>>(&mut self, destination: &mut [S]) {
        let main_output = &self.output_port_channels
            [self.config.plugin_output_port_config.main_port_index as usize];
        let muxed = &mut self.muxed[..destination.len()];

        let plugin_output_channel_count = self
            .config
            .plugin_output_port_config
            .main_port()
            .port_layout
            .channel_count();

        match (
            plugin_output_channel_count,
            self.config.output_channel_count,
        ) {
            // Mono-to-mono: we do a simple copy
            (1, 1) => muxed.copy_from_slice(main_output),
            // Stereo (or anything)-to-mono: we'll mix all the channels down to mono
            (n, 1) => mix_mono(main_output, muxed, n as usize),
            // Mono-to-Stereo: use the same samples for each channel
            (1, 2) => mono_to_multi(main_output, muxed, 2),
            // Stereo-to-stereo: Mux. If there are more source channels, we'll take only the first two.
            (_, 2) => mux(main_output, muxed, 2),

            (_, _) => unreachable!(),
        }

        for (out, muxed) in destination.iter_mut().zip(muxed) {
            *out = muxed.to_sample();
        }
    }
}

fn mux(channels_buffer: &[f32], output: &mut [f32], channel_count: usize) {
    assert!(channels_buffer.len() >= output.len());

    // Probably not the best implementation, but it works
    let single_channel_len = channels_buffer.len() / channel_count;
    for (muxed_index, output_sample) in output.iter_mut().enumerate() {
        let channel_number = muxed_index % channel_count;
        let channel_buffer_index = muxed_index / channel_count;
        let position = (channel_number * single_channel_len) + channel_buffer_index;

        *output_sample = channels_buffer[position]
    }
}

fn mix_mono(channels_buffer: &[f32], mono_output: &mut [f32], channel_count: usize) {
    assert!(channel_count > 0);
    assert!(channels_buffer.len() >= mono_output.len() * channel_count);

    let single_channel_len = channels_buffer.len() / channel_count;
    for (index, output_sample) in mono_output.iter_mut().enumerate() {
        let mut total = 0.0;
        for channel_number in 0..channel_count {
            let position = (channel_number * single_channel_len) + index;
            total += channels_buffer[position]
        }
        *output_sample = total / (channel_count as f32);
    }
}

fn mono_to_multi(mono_input: &[f32], multi_output: &mut [f32], channel_count: usize) {
    assert!(channel_count > 0);

    for (output_samples, input_sample) in
        multi_output.chunks_exact_mut(channel_count).zip(mono_input)
    {
        output_samples.fill(*input_sample)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mux_works() {
        let channels = [1.0, 2.0, 3.0, 1.5, 2.5, 3.5]; // L then R channel, sequentially
        let mut muxed = [0.0; 6];

        mux(&channels, &mut muxed, 2);

        assert_eq!(muxed, [1.0, 1.5, 2.0, 2.5, 3.0, 3.5])
    }
}
