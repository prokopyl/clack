use crate::process::audio::SampleType;
use clack_common::process::ConstantMask;
use clap_sys::audio_buffer::clap_audio_buffer;

#[derive(Copy, Clone)]
pub struct InputPort<'a> {
    inner: &'a clap_audio_buffer,
    frames_count: u32,
}

pub(crate) fn buffer_is_f32(buf: &clap_audio_buffer) -> bool {
    match (buf.data32.is_null(), buf.data64.is_null()) {
        (false, _) => true,
        (true, false) => false,
        _ => panic!(
            "Invalid audio buffer data (both data32 and data64 pointers are either null or set)"
        ),
    }
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
    pub fn channels(&self) -> SampleType<InputChannels<'a, f32>, InputChannels<'a, f64>> {
        unsafe {
            // TODO: use proper method
            if buffer_is_f32(self.inner) {
                SampleType::F32(InputChannels {
                    data: core::slice::from_raw_parts(
                        self.inner.data32 as *const _,
                        self.inner.channel_count as usize,
                    ),
                    frames_count: self.frames_count,
                })
            } else {
                SampleType::F64(InputChannels {
                    data: core::slice::from_raw_parts(
                        self.inner.data64 as *const _,
                        self.inner.channel_count as usize,
                    ),
                    frames_count: self.frames_count,
                })
            }
        }
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
    pub(crate) frames_count: u32,
    pub(crate) data: &'a [*const S],
}

impl<'a, S> InputChannels<'a, S> {
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.data.len() as u32
    }

    #[inline]
    pub fn get_channel_data(&self, channel_index: u32) -> Option<&'a [S]> {
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
    pub(crate) data: core::slice::Iter<'a, *const T>,
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
}
