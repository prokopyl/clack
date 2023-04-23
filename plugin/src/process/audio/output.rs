use crate::prelude::Audio;
use crate::process::audio::SampleType;
use crate::process::InputChannelsIter;
use clack_common::process::ConstantMask;
use clap_sys::audio_buffer::clap_audio_buffer;
use std::slice::IterMut;

pub struct OutputPortsIter<'a> {
    outputs: IterMut<'a, clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> OutputPortsIter<'a> {
    #[inline]
    pub(crate) fn new(audio: &'a mut Audio<'_>) -> Self {
        Self {
            outputs: audio.outputs.iter_mut(),
            frames_count: audio.frames_count,
        }
    }
}

impl<'a> Iterator for OutputPortsIter<'a> {
    type Item = OutputPort<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.outputs
            .next()
            .map(|buf| unsafe { OutputPort::from_raw(buf, self.frames_count) })
    }
}

pub struct OutputPort<'a> {
    inner: &'a mut clap_audio_buffer,
    frames_count: u32,
}

impl<'a> OutputPort<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(inner: &'a mut clap_audio_buffer, frames_count: u32) -> Self {
        Self {
            inner,
            frames_count,
        }
    }

    #[inline]
    pub fn channels(
        &mut self,
    ) -> Option<SampleType<OutputChannels<'a, f32>, OutputChannels<'a, f64>>> {
        Some(unsafe { SampleType::from_raw_buffer_mut(self.inner) }?.map(
            |data| OutputChannels {
                data,
                frames_count: self.frames_count,
            },
            |data| OutputChannels {
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

    #[inline]
    pub fn set_constant_mask(&mut self, new_mask: ConstantMask) {
        self.inner.constant_mask = new_mask.to_bits()
    }
}

pub struct OutputChannels<'a, S> {
    pub(crate) frames_count: u32,
    pub(crate) data: &'a [*const S],
}

impl<'a, S> OutputChannels<'a, S> {
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.data.len() as u32
    }

    #[inline]
    pub fn channel(&self, channel_index: u32) -> Option<&'a [S]> {
        unsafe {
            self.data.get(channel_index as usize).map(|data| {
                core::slice::from_raw_parts(*data as *const _, self.frames_count as usize)
            })
        }
    }

    #[inline]
    pub fn channel_mut(&mut self, channel_index: u32) -> Option<&'a mut [S]> {
        unsafe {
            self.data.get(channel_index as usize).map(|data| {
                core::slice::from_raw_parts_mut(*data as *mut _, self.frames_count as usize)
            })
        }
    }

    #[inline]
    pub fn iter(&self) -> InputChannelsIter<'a, S> {
        InputChannelsIter {
            data: self.data.iter(),
            frames_count: self.frames_count,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> OutputChannelsIter<S> {
        OutputChannelsIter {
            data: self.data.as_ref().iter(),
            frames_count: self.frames_count,
        }
    }
}
impl<'a, T> IntoIterator for &'a OutputChannels<'a, T> {
    type Item = &'a [T];
    type IntoIter = InputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut OutputChannels<'a, T> {
    type Item = &'a mut [T];
    type IntoIter = OutputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for OutputChannels<'a, T> {
    type Item = &'a mut [T];
    type IntoIter = OutputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        OutputChannelsIter {
            data: self.data.as_ref().iter(),
            frames_count: self.frames_count,
        }
    }
}

pub struct OutputChannelsIter<'a, T> {
    data: core::slice::Iter<'a, *const T>,
    frames_count: u32,
}

impl<'a, T> Iterator for OutputChannelsIter<'a, T> {
    type Item = &'a mut [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|ptr| unsafe {
            core::slice::from_raw_parts_mut(*ptr as *mut _, self.frames_count as usize)
        })
    }
}
