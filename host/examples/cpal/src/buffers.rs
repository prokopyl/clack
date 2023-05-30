use clack_host::prelude::{
    AudioPortBuffer, AudioPortBufferType, AudioPorts, InputAudioBuffers, InputChannel,
    OutputAudioBuffers,
};
use cpal::{FromSample, Sample};

pub struct CpalAudioOutputBuffers {
    output_ports: AudioPorts,
    input_ports: AudioPorts,
    output_channels: Vec<f32>,
    input_channels: Vec<f32>,
    muxed: Vec<f32>,
    channel_count: usize,
    frame_count: usize,
}

impl CpalAudioOutputBuffers {
    pub fn with_capacity(channel_count: usize, frame_count: usize) -> Self {
        Self {
            input_ports: AudioPorts::with_capacity(channel_count, 1),
            output_ports: AudioPorts::with_capacity(channel_count, 1),
            output_channels: vec![0.0; frame_count * channel_count],
            input_channels: vec![0.0; frame_count * channel_count],
            muxed: vec![0.0; frame_count * channel_count],
            channel_count,
            frame_count,
        }
    }

    pub fn ensure_buffer_size_matches(&mut self, total_buffer_size: usize) {
        if self.input_channels.len() != total_buffer_size {
            self.input_channels.resize(total_buffer_size, 0.0);
        }

        if self.output_channels.len() != total_buffer_size {
            self.output_channels.resize(total_buffer_size, 0.0);
        }

        if self.muxed.len() != total_buffer_size {
            self.muxed.resize(total_buffer_size, 0.0);
        }

        self.frame_count = total_buffer_size / self.channel_count;
    }

    pub fn plugin_buffers(&mut self) -> (InputAudioBuffers, OutputAudioBuffers) {
        self.output_channels.fill(0.0); // just in case
        self.input_channels.fill(0.0); // just in case

        (
            self.input_ports.with_input_buffers([AudioPortBuffer {
                latency: 0,
                channels: AudioPortBufferType::f32_input_only(
                    self.input_channels
                        .chunks_exact_mut(self.frame_count)
                        .map(|buffer| InputChannel {
                            buffer,
                            is_constant: true,
                        }),
                ),
            }]),
            self.output_ports.with_output_buffers([AudioPortBuffer {
                latency: 0,
                channels: AudioPortBufferType::f32_output_only(
                    self.output_channels.chunks_exact_mut(self.frame_count),
                ),
            }]),
        )
    }

    pub fn write_to<S: FromSample<f32>>(&mut self, destination: &mut [S]) {
        mux(&self.output_channels, &mut self.muxed, self.channel_count);

        for (out, muxed) in destination.iter_mut().zip(&self.muxed) {
            *out = muxed.to_sample();
        }
    }
}

fn mux(channels_buffer: &[f32], output: &mut [f32], channel_count: usize) {
    assert_eq!(channels_buffer.len(), output.len());

    // Probably not the best implementation, but it works
    let single_channel_len = channels_buffer.len() / channel_count;
    for (muxed_index, output_sample) in output.iter_mut().enumerate() {
        let channel_number = muxed_index % channel_count;
        let channel_buffer_index = muxed_index / channel_count;
        let position = (channel_number * single_channel_len) + channel_buffer_index;

        *output_sample = channels_buffer[position]
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
