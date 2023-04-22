use crate::prelude::Audio;
use crate::process::audio::channels::ChannelPair::{InputOnly, OutputOnly};
use crate::process::audio::channels::{ChannelPair, SampleType};
use crate::process::audio::port::{AudioInputPort, AudioOutputPort};
use clap_sys::audio_buffer::clap_audio_buffer;
use std::slice::{Iter, IterMut};

pub struct AudioPortPair<'a> {
    input: Option<&'a clap_audio_buffer>,
    output: Option<&'a mut clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> AudioPortPair<'a> {
    #[inline]
    pub fn input(&self) -> Option<AudioInputPort<'a>> {
        // SAFETY: TODO
        self.input
            .map(|i| unsafe { AudioInputPort::from_raw(i, self.frames_count) })
    }

    #[inline]
    pub fn output(&mut self) -> Option<AudioOutputPort> {
        // SAFETY: TODO
        self.output
            .as_mut()
            .map(|i| unsafe { AudioOutputPort::from_raw(i, self.frames_count) })
    }

    fn input_channel(&self, index: usize) -> Option<SampleType<&[f32], &[f64]>> {
        todo!()
    }

    fn output_channel(&self, index: usize) -> Option<SampleType<&mut [f32], &mut [f64]>> {
        todo!()
    }

    pub fn channel_pair(
        &mut self,
        index: usize,
    ) -> Option<SampleType<ChannelPair<f32>, ChannelPair<f64>>> {
        let input = self.input_channel(index);
        let output = self.output_channel(index);

        channel_pair_from_io(input, output)
    }

    pub fn channel_pairs(
        &mut self,
    ) -> Option<SampleType<AudioChannelPairsIter<'a, f32>, AudioChannelPairsIter<'a, f64>>> {
        let input = self
            .input
            .and_then(|b| unsafe { SampleType::from_raw_buffer(b) })
            .map_or_else(
                || SampleType::Both([].iter(), [].iter()),
                |s| s.map(|b| b.iter(), |b| b.iter()),
            );

        let output = self
            .output
            .as_mut()
            .and_then(|b| unsafe { SampleType::from_raw_buffer_mut(b) })
            .map_or_else(
                || SampleType::Both([].iter_mut(), [].iter_mut()),
                |s| s.map(|b| b.iter_mut(), |b| b.iter_mut()),
            );

        input.try_match_with(output).map(|s| {
            s.map(
                |(i, o)| AudioChannelPairsIter {
                    input_iter: i,
                    output_iter: o,
                    frames_count: self.frames_count,
                },
                |(i, o)| AudioChannelPairsIter {
                    input_iter: i,
                    output_iter: o,
                    frames_count: self.frames_count,
                },
            )
        })
    }
}

pub struct AudioChannelPairsIter<'a, S> {
    input_iter: Iter<'a, *const S>,
    output_iter: IterMut<'a, *const S>,
    frames_count: u32,
}

impl<'a, S> Iterator for AudioChannelPairsIter<'a, S> {
    type Item = ChannelPair<'a, S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let input = self
            .input_iter
            .next()
            .map(|ptr| unsafe { core::slice::from_raw_parts(*ptr, self.frames_count as usize) });

        let output = self.output_iter.next().map(|ptr| unsafe {
            core::slice::from_raw_parts_mut((*ptr) as *mut _, self.frames_count as usize)
        });

        channel_pair_from_io_2(input, output)
    }
}

// TODO: inline this into ChannelPair
#[inline]
fn channel_pair_from_io_2<'a, S>(
    input: Option<&'a [S]>,
    output: Option<&'a mut [S]>,
) -> Option<ChannelPair<'a, S>> {
    match (input, output) {
        (None, None) => None,
        (Some(input), None) => Some(InputOnly(input)),
        (None, Some(output)) => Some(OutputOnly(output)),
        (Some(input), Some(output)) => Some(ChannelPair::from_io(input, output)),
    }
}

// TODO: remove this
#[inline]
fn channel_pair_from_io<'a>(
    input: Option<SampleType<&'a [f32], &'a [f64]>>,
    output: Option<SampleType<&'a mut [f32], &'a mut [f64]>>,
) -> Option<SampleType<ChannelPair<'a, f32>, ChannelPair<'a, f64>>> {
    match (input, output) {
        (None, None) => None,
        (Some(input), None) => Some(input.map(InputOnly, InputOnly)),
        (None, Some(output)) => Some(output.map(OutputOnly, OutputOnly)),
        (Some(input), Some(output)) => Some(input.try_match_with(output)?.map(
            |(i, o)| ChannelPair::from_io(i, o),
            |(i, o)| ChannelPair::from_io(i, o),
        )),
    }
}

pub struct AudioPortsPairIter<'a> {
    inputs: Iter<'a, clap_audio_buffer>,
    outputs: IterMut<'a, clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> AudioPortsPairIter<'a> {
    #[inline]
    pub(crate) fn new(audio: &'a mut Audio<'_>) -> Self {
        Self {
            inputs: audio.inputs.iter(),
            outputs: audio.outputs.iter_mut(),
            frames_count: audio.frames_count,
        }
    }
}

impl<'a> Iterator for AudioPortsPairIter<'a> {
    type Item = AudioPortPair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.inputs.next(), self.outputs.next()) {
            (None, None) => None,
            (input, output) => Some(AudioPortPair {
                input,
                output,
                frames_count: self.frames_count,
            }),
        }
    }
}
