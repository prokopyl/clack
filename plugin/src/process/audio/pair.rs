use crate::process::audio::pair::ChannelPair::*;
use crate::process::audio::{InputPort, OutputPort, SampleType};
use crate::process::Audio;
use clap_sys::audio_buffer::clap_audio_buffer;
use std::slice::{Iter, IterMut};

pub struct PortPair<'a> {
    input: Option<&'a clap_audio_buffer>,
    output: Option<&'a mut clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> PortPair<'a> {
    #[inline]
    pub fn input(&self) -> Option<InputPort<'a>> {
        self.input
            .map(|i| unsafe { InputPort::from_raw(i, self.frames_count) })
    }

    #[inline]
    pub fn output(&mut self) -> Option<OutputPort> {
        self.output
            .as_mut()
            .map(|i| unsafe { OutputPort::from_raw(i, self.frames_count) })
    }

    #[inline]
    fn input_channel(&self, index: usize) -> Option<SampleType<&'a [f32], &'a [f64]>> {
        Some(
            unsafe { SampleType::from_raw_buffer(self.input?) }?
                .map_option(|b| b.get(index), |b| b.get(index))?
                .map(
                    |b| unsafe { core::slice::from_raw_parts(*b, self.frames_count as usize) },
                    |b| unsafe { core::slice::from_raw_parts(*b, self.frames_count as usize) },
                ),
        )
    }

    #[inline]
    fn output_channel(&mut self, index: usize) -> Option<SampleType<&'a mut [f32], &'a mut [f64]>> {
        Some(
            unsafe { SampleType::from_raw_buffer_mut(self.output.as_mut()?) }?
                .map_option(|b| b.get(index), |b| b.get(index))?
                .map(
                    |b| unsafe {
                        core::slice::from_raw_parts_mut(*b as *mut _, self.frames_count as usize)
                    },
                    |b| unsafe {
                        core::slice::from_raw_parts_mut(*b as *mut _, self.frames_count as usize)
                    },
                ),
        )
    }

    pub fn channel_pair(
        &mut self,
        index: usize,
    ) -> Option<SampleType<ChannelPair<'a, f32>, ChannelPair<'a, f64>>> {
        let input = self.input_channel(index);
        let output = self.output_channel(index);

        channel_pair_from_io(input, output)
    }

    pub fn channel_pairs(
        &mut self,
    ) -> Option<SampleType<ChannelPairsIter<'a, f32>, ChannelPairsIter<'a, f64>>> {
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
                |(i, o)| ChannelPairsIter {
                    input_iter: i,
                    output_iter: o,
                    frames_count: self.frames_count,
                },
                |(i, o)| ChannelPairsIter {
                    input_iter: i,
                    output_iter: o,
                    frames_count: self.frames_count,
                },
            )
        })
    }
}

pub struct ChannelPairsIter<'a, S> {
    input_iter: Iter<'a, *const S>,
    output_iter: IterMut<'a, *const S>,
    frames_count: u32,
}

impl<'a, S> Iterator for ChannelPairsIter<'a, S> {
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

pub struct PortsPairIter<'a> {
    inputs: Iter<'a, clap_audio_buffer>,
    outputs: IterMut<'a, clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> PortsPairIter<'a> {
    #[inline]
    pub(crate) fn new(audio: &'a mut Audio<'_>) -> Self {
        Self {
            inputs: audio.inputs.iter(),
            outputs: audio.outputs.iter_mut(),
            frames_count: audio.frames_count,
        }
    }
}

impl<'a> Iterator for PortsPairIter<'a> {
    type Item = PortPair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.inputs.next(), self.outputs.next()) {
            (None, None) => None,
            (input, output) => Some(PortPair {
                input,
                output,
                frames_count: self.frames_count,
            }),
        }
    }
}

pub enum ChannelPair<'a, S> {
    InputOnly(&'a [S]),
    OutputOnly(&'a mut [S]),
    InputOutput(&'a [S], &'a mut [S]),
    InPlace(&'a mut [S]),
}

impl<'a, S> ChannelPair<'a, S> {
    #[inline]
    pub(crate) fn from_io(input: &'a [S], output: &'a mut [S]) -> Self {
        if input.as_ptr() == output.as_ptr() {
            InPlace(output)
        } else {
            InputOutput(input, output)
        }
    }

    #[inline]
    pub fn input(&'a self) -> Option<&'a [S]> {
        match self {
            InputOnly(i) | InputOutput(i, _) => Some(i),
            OutputOnly(_) => None,
            InPlace(io) => Some(io),
        }
    }

    #[inline]
    pub fn output(&'a self) -> Option<&'a [S]> {
        match self {
            OutputOnly(o) | InputOutput(_, o) | InPlace(o) => Some(o),
            InputOnly(_) => None,
        }
    }
}
