use crate::process::audio::channels::{AudioBufferType, TAudioChannels, TAudioChannelsMut};
use clap_sys::audio_buffer::clap_audio_buffer;

#[derive(Copy, Clone)]
pub struct AudioBuffer<'a> {
    inner: &'a clap_audio_buffer,
    frames_count: u32,
}

pub(crate) fn buffer_is_f32(buf: &clap_audio_buffer) -> bool {
    match (buf.data32.is_null(), buf.data64.is_null()) {
        (false, true) => true,
        (true, false) => false,
        _ => panic!(
            "Invalid audio buffer data (both data32 and data64 pointers are either null or set)"
        ),
    }
}

impl<'a> AudioBuffer<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(inner: &'a clap_audio_buffer, frames_count: u32) -> Self {
        Self {
            inner,
            frames_count,
        }
    }

    #[inline]
    pub fn channels(&self) -> AudioBufferType<TAudioChannels<'a, f32>, TAudioChannels<'a, f64>> {
        unsafe {
            if buffer_is_f32(self.inner) {
                AudioBufferType::F32(TAudioChannels {
                    data: ::core::slice::from_raw_parts(
                        self.inner.data32 as *const _,
                        self.inner.channel_count as usize,
                    ),
                    frames_count: self.frames_count,
                })
            } else {
                AudioBufferType::F64(TAudioChannels {
                    data: ::core::slice::from_raw_parts(
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
    pub fn is_constant(&self, channel_index: u32) -> bool {
        (self.inner.constant_mask & (1 << channel_index as u64)) == 1
    }

    #[inline]
    pub fn constant_mask(&self) -> u64 {
        self.inner.constant_mask
    }
}

pub struct AudioBufferMut<'a> {
    inner: &'a clap_audio_buffer,
    frames_count: u32,
}

impl<'a> AudioBufferMut<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(inner: &'a clap_audio_buffer, frames_count: u32) -> Self {
        Self {
            inner,
            frames_count,
        }
    }

    #[inline]
    pub fn channels_mut(
        &mut self,
    ) -> AudioBufferType<TAudioChannelsMut<'a, f32>, TAudioChannelsMut<'a, f64>> {
        unsafe {
            if buffer_is_f32(self.inner) {
                AudioBufferType::F32(TAudioChannelsMut {
                    data: ::core::slice::from_raw_parts(
                        self.inner.data32 as *const _,
                        self.inner.channel_count as usize,
                    ),
                    frames_count: self.frames_count,
                })
            } else {
                AudioBufferType::F64(TAudioChannelsMut {
                    data: ::core::slice::from_raw_parts(
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
    pub fn is_constant(&self, channel_index: u32) -> bool {
        (self.inner.constant_mask & (1 << channel_index as u64)) == 1
    }

    #[inline]
    pub fn constant_mask(&self) -> u64 {
        self.inner.constant_mask
    }
}
