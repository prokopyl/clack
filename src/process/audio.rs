use crate::process::audio::buffer::{AudioBuffer, AudioBufferMut};
use crate::process::audio::channels::{AudioBufferType, TAudioChannelsMut};
use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::process::clap_process;
use std::cell::Cell;

pub struct Audio<'a> {
    inputs: &'a mut [clap_audio_buffer],
    outputs: &'a mut [clap_audio_buffer],
    frames_count: u32,
}

impl<'a> Audio<'a> {
    #[inline]
    pub(crate) fn from_raw(process: &clap_process) -> Audio {
        unsafe {
            Audio {
                frames_count: process.frames_count,
                inputs: ::core::slice::from_raw_parts_mut(
                    process.audio_inputs as *mut _,
                    process.audio_inputs_count as usize,
                ),
                outputs: ::core::slice::from_raw_parts_mut(
                    process.audio_outputs as *mut _,
                    process.audio_outputs_count as usize,
                ),
            }
        }
    }
    pub fn input(&self, index: usize) -> Option<AudioBuffer> {
        self.inputs
            .get(index)
            .map(|buf| unsafe { AudioBuffer::from_raw(buf, self.frames_count) })
    }

    pub fn output(&mut self, index: usize) -> Option<AudioBufferMut> {
        self.outputs
            .get_mut(index)
            .map(|buf| unsafe { AudioBufferMut::from_raw(buf, self.frames_count) })
    }

    fn zip_channels<'b, T: Sized>(
        input: &mut TAudioChannelsMut<'b, T>,
        output: &mut TAudioChannelsMut<'b, T>,
        channel_index: usize,
    ) -> Option<impl Iterator<Item = (&'b Cell<T>, &'b Cell<T>)>> {
        let input = input.get_channel_data_mut(channel_index)?;
        let output = output.get_channel_data_mut(channel_index)?;

        Some(
            Cell::from_mut(input)
                .as_slice_of_cells()
                .iter()
                .zip(Cell::from_mut(output).as_slice_of_cells().iter()),
        )
    }

    pub fn zip(
        &mut self,
        port_index: usize,
        channel_index: usize,
    ) -> Option<
        AudioBufferType<
            impl Iterator<Item = (&Cell<f32>, &Cell<f32>)>,
            impl Iterator<Item = (&Cell<f64>, &Cell<f64>)>,
        >,
    > {
        let mut input_buffer = unsafe {
            AudioBufferMut::from_raw(self.inputs.get_mut(port_index)?, self.frames_count)
        };

        let mut output_buffer = unsafe {
            AudioBufferMut::from_raw(self.outputs.get_mut(port_index)?, self.frames_count)
        };

        match (input_buffer.channels_mut(), output_buffer.channels_mut()) {
            (AudioBufferType::F32(mut in_chans), AudioBufferType::F32(mut out_chans)) => {
                Some(AudioBufferType::F32(Self::zip_channels(&mut in_chans, &mut out_chans, channel_index)?))
            },
            (AudioBufferType::F64(mut in_chans), AudioBufferType::F64(mut out_chans)) =>
                Some(AudioBufferType::F64(Self::zip_channels(&mut in_chans, &mut out_chans, channel_index)?)),
            _ => panic!("Cannot reconciliate buffer types: input and output buffers must be both either f32 or f64")
        }
    }
}

pub mod buffer;
pub mod channels;
