use crate::process::audio::pair::ChannelPair::*;
use crate::process::audio::{BufferError, InputPort, OutputPort, SampleType};
use crate::process::Audio;
use clap_sys::audio_buffer::clap_audio_buffer;
use std::slice::{Iter, IterMut};

pub struct PairedPort<'a> {
    input: Option<&'a clap_audio_buffer>,
    output: Option<&'a mut clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> PairedPort<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(
        input: Option<&'a clap_audio_buffer>,
        output: Option<&'a mut clap_audio_buffer>,
        frames_count: u32,
    ) -> Option<Self> {
        match (input, output) {
            (None, None) => None,
            (input, output) => Some(PairedPort {
                input,
                output,
                frames_count,
            }),
        }
    }

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
    pub fn channels(
        &mut self,
    ) -> Result<SampleType<PairedChannels<'a, f32>, PairedChannels<'a, f64>>, BufferError> {
        let input = match self.input {
            None => SampleType::Both([].as_slice(), [].as_slice()),
            Some(buffer) => unsafe { SampleType::from_raw_buffer(buffer)? },
        };

        let output = match self.output.as_mut() {
            None => SampleType::Both([].as_mut_slice(), [].as_mut_slice()),
            Some(buffer) => unsafe { SampleType::from_raw_buffer_mut(buffer)? },
        };

        Ok(input.try_match_with(output)?.map(
            |(i, o)| PairedChannels {
                input_data: i,
                output_data: o,
                frames_count: self.frames_count,
            },
            |(i, o)| PairedChannels {
                input_data: i,
                output_data: o,
                frames_count: self.frames_count,
            },
        ))
    }

    #[inline]
    pub fn channel_pair_count(&self) -> usize {
        let in_channels = self.input.map(|b| b.channel_count).unwrap_or(0);
        let out_channels = self.output.as_ref().map(|b| b.channel_count).unwrap_or(0);

        in_channels.max(out_channels) as usize
    }
}

pub struct PairedChannels<'a, S> {
    input_data: &'a [*const S],
    output_data: &'a mut [*const S],
    frames_count: u32,
}

impl<'a, S> PairedChannels<'a, S> {
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    #[inline]
    pub fn input_channel_count(&self) -> usize {
        self.input_data.len()
    }

    #[inline]
    pub fn output_channel_count(&self) -> usize {
        self.output_data.len()
    }

    #[inline]
    pub fn channel_pair_count(&self) -> usize {
        self.input_channel_count().max(self.output_channel_count())
    }

    #[inline]
    pub fn channel_pair(&mut self, index: usize) -> Option<ChannelPair<'a, S>> {
        self.mismatched_channel_pair(index, index)
    }

    #[inline]
    pub fn mismatched_channel_pair(
        &mut self,
        input_index: usize,
        output_index: usize,
    ) -> Option<ChannelPair<'a, S>> {
        let input = self
            .input_data
            .get(input_index)
            .map(|ptr| unsafe { core::slice::from_raw_parts(*ptr, self.frames_count as usize) });
        let output = self.output_data.get(output_index).map(|ptr| unsafe {
            core::slice::from_raw_parts_mut(*ptr as *mut _, self.frames_count as usize)
        });

        ChannelPair::from_optional_io(input, output)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> PairedChannelsIter<S> {
        PairedChannelsIter {
            input_iter: self.input_data.iter(),
            output_iter: self.output_data.iter_mut(),
            frames_count: self.frames_count,
        }
    }
}

impl<'a, S> IntoIterator for PairedChannels<'a, S> {
    type Item = ChannelPair<'a, S>;
    type IntoIter = PairedChannelsIter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        PairedChannelsIter {
            input_iter: self.input_data.iter(),
            output_iter: self.output_data.iter_mut(),
            frames_count: self.frames_count,
        }
    }
}

pub struct PairedChannelsIter<'a, S> {
    input_iter: Iter<'a, *const S>,
    output_iter: IterMut<'a, *const S>,
    frames_count: u32,
}

impl<'a, S> Iterator for PairedChannelsIter<'a, S> {
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

        ChannelPair::from_optional_io(input, output)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, S> ExactSizeIterator for PairedChannelsIter<'a, S> {
    #[inline]
    fn len(&self) -> usize {
        self.input_iter.len().max(self.output_iter.len())
    }
}

pub struct PairedPortsIter<'a> {
    inputs: Iter<'a, clap_audio_buffer>,
    outputs: IterMut<'a, clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> PairedPortsIter<'a> {
    #[inline]
    pub(crate) fn new(audio: &'a mut Audio<'_>) -> Self {
        Self {
            inputs: audio.inputs.iter(),
            outputs: audio.outputs.iter_mut(),
            frames_count: audio.frames_count,
        }
    }
}

impl<'a> Iterator for PairedPortsIter<'a> {
    type Item = PairedPort<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { PairedPort::from_raw(self.inputs.next(), self.outputs.next(), self.frames_count) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> ExactSizeIterator for PairedPortsIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inputs.len().max(self.outputs.len())
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
    pub(crate) fn from_optional_io(
        input: Option<&'a [S]>,
        output: Option<&'a mut [S]>,
    ) -> Option<ChannelPair<'a, S>> {
        match (input, output) {
            (None, None) => None,
            (Some(input), None) => Some(InputOnly(input)),
            (None, Some(output)) => Some(OutputOnly(output)),
            (Some(input), Some(output)) => Some(if input.as_ptr() == output.as_ptr() {
                InPlace(output)
            } else {
                InputOutput(input, output)
            }),
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
