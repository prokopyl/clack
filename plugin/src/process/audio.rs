use crate::process::audio::pair::{AudioPortPair, AudioPortsPairIter};
use crate::process::audio::port::{AudioInputPort, AudioOutputPort};
use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::process::clap_process;

pub mod channels;
pub mod pair;
pub mod port;

pub struct Audio<'a> {
    inputs: &'a [clap_audio_buffer],
    outputs: &'a mut [clap_audio_buffer],
    frames_count: u32,
}

impl<'a> Audio<'a> {
    #[inline]
    pub(crate) fn from_raw(process: &clap_process) -> Audio {
        unsafe {
            Audio {
                frames_count: process.frames_count,
                inputs: core::slice::from_raw_parts(
                    process.audio_inputs,
                    process.audio_inputs_count as usize,
                ),
                outputs: core::slice::from_raw_parts_mut(
                    process.audio_outputs,
                    process.audio_outputs_count as usize,
                ),
            }
        }
    }

    pub fn input(&self, index: usize) -> Option<AudioInputPort> {
        self.inputs
            .get(index)
            .map(|buf| unsafe { AudioInputPort::from_raw(buf, self.frames_count) })
    }

    #[inline]
    pub fn input_count(self) -> usize {
        self.inputs.len()
    }

    #[inline]
    pub fn output(&mut self, index: usize) -> Option<AudioOutputPort> {
        self.outputs
            .get_mut(index)
            // SAFETY: &mut ensures there is no input being read concurrently
            .map(|buf| unsafe { AudioOutputPort::from_raw(buf, self.frames_count) })
    }

    #[inline]
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    #[inline]
    pub fn port_pairs(&mut self) -> AudioPortsPairIter {
        AudioPortsPairIter::new(self)
    }
}

impl<'a> IntoIterator for &'a mut Audio<'a> {
    type Item = AudioPortPair<'a>;
    type IntoIter = AudioPortsPairIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.port_pairs()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use clack_host::prelude::*;

    #[test]
    fn can_get_all_outputs() {
        let ins = [[0f32; 4]; 2];
        let mut outs = [[0f32; 4]; 2];

        let mut audio = Audio {
            inputs: &[clap_audio_buffer {
                data32: &ins as *const _ as *const _,
                data64: ::core::ptr::null(),
                constant_mask: 0,
                latency: 0,
                channel_count: 2,
            }],
            outputs: &mut [clap_audio_buffer {
                data32: &mut outs as *const _ as *const _,
                data64: ::core::ptr::null(),
                constant_mask: 0,
                latency: 0,
                channel_count: 2,
            }],
            frames_count: 4,
        };

        let pairs = audio.port_pairs().collect::<Vec<_>>();
    }
}
