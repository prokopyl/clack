use crate::prelude::Audio;
use crate::process::audio::SampleType;
use clack_common::process::ConstantMask;
use clap_sys::audio_buffer::clap_audio_buffer;
use std::slice::Iter;

pub struct InputPortsIter<'a> {
    inputs: Iter<'a, clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> InputPortsIter<'a> {
    #[inline]
    pub(crate) fn new(audio: &Audio<'a>) -> Self {
        Self {
            inputs: audio.inputs.iter(),
            frames_count: audio.frames_count,
        }
    }
}

impl<'a> Iterator for InputPortsIter<'a> {
    type Item = InputPort<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inputs
            .next()
            .map(|buf| unsafe { InputPort::from_raw(buf, self.frames_count) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inputs.size_hint()
    }
}

impl<'a> ExactSizeIterator for InputPortsIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inputs.len()
    }
}

#[derive(Copy, Clone)]
pub struct InputPort<'a> {
    inner: &'a clap_audio_buffer,
    frames_count: u32,
}

impl<'a> InputPort<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(inner: &'a clap_audio_buffer, frames_count: u32) -> Self {
        Self {
            inner,
            frames_count,
        }
    }

    #[inline]
    pub fn channels(&self) -> Option<SampleType<InputChannels<'a, f32>, InputChannels<'a, f64>>> {
        Some(unsafe { SampleType::from_raw_buffer(self.inner) }?.map(
            |data| InputChannels {
                data,
                frames_count: self.frames_count,
            },
            |data| InputChannels {
                data,
                frames_count: self.frames_count,
            },
        ))
    }

    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.inner.channel_count
    }

    #[inline]
    pub fn latency(&self) -> u32 {
        self.inner.latency
    }

    #[inline]
    pub fn constant_mask(&self) -> ConstantMask {
        ConstantMask::from_bits(self.inner.constant_mask)
    }
}

#[derive(Copy, Clone)]
pub struct InputChannels<'a, S> {
    frames_count: u32,
    data: &'a [*const S],
}

impl<'a, S> InputChannels<'a, S> {
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    #[inline]
    pub fn raw_data(&self) -> &'a [*const S] {
        self.data
    }

    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.data.len() as u32
    }

    #[inline]
    pub fn channel(&self, channel_index: u32) -> Option<&'a [S]> {
        unsafe {
            self.data
                .get(channel_index as usize)
                .map(|data| core::slice::from_raw_parts(*data, self.frames_count as usize))
        }
    }

    #[inline]
    pub fn iter(&self) -> InputChannelsIter<'a, S> {
        InputChannelsIter {
            data: self.data.iter(),
            frames_count: self.frames_count,
        }
    }
}

impl<'a, T> IntoIterator for InputChannels<'a, T> {
    type Item = &'a [T];
    type IntoIter = InputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a InputChannels<'a, T> {
    type Item = &'a [T];
    type IntoIter = InputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct InputChannelsIter<'a, T> {
    // TODO: hide these with new() function
    pub(crate) data: Iter<'a, *const T>,
    pub(crate) frames_count: u32,
}

impl<'a, T> Iterator for InputChannelsIter<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.data
            .next()
            .map(|ptr| unsafe { core::slice::from_raw_parts(*ptr, self.frames_count as usize) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.data.size_hint()
    }
}

impl<'a, S> ExactSizeIterator for InputChannelsIter<'a, S> {
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}
