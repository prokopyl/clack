use crate::process::audio::channels::{SampleType, TAudioChannels, TAudioChannelsMut};
use clack_common::process::ConstantMask;
use clap_sys::audio_buffer::clap_audio_buffer;

#[derive(Copy, Clone)]
pub struct AudioInputPort<'a> {
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

impl<'a> AudioInputPort<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(inner: &'a clap_audio_buffer, frames_count: u32) -> Self {
        Self {
            inner,
            frames_count,
        }
    }

    #[inline]
    pub fn channels(&self) -> SampleType<TAudioChannels<'a, f32>, TAudioChannels<'a, f64>> {
        unsafe {
            if buffer_is_f32(self.inner) {
                SampleType::F32(TAudioChannels {
                    data: core::slice::from_raw_parts(
                        self.inner.data32 as *const _,
                        self.inner.channel_count as usize,
                    ),
                    frames_count: self.frames_count,
                })
            } else {
                SampleType::F64(TAudioChannels {
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

pub struct AudioOutputPort<'a> {
    inner: &'a mut clap_audio_buffer,
    frames_count: u32,
}

impl<'a> AudioOutputPort<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(inner: &'a mut clap_audio_buffer, frames_count: u32) -> Self {
        Self {
            inner,
            frames_count,
        }
    }

    #[inline]
    pub fn channels_mut(
        &mut self,
    ) -> SampleType<TAudioChannelsMut<'a, f32>, TAudioChannelsMut<'a, f64>> {
        unsafe {
            if buffer_is_f32(self.inner) {
                SampleType::F32(TAudioChannelsMut {
                    data: core::slice::from_raw_parts(
                        self.inner.data32 as *const _,
                        self.inner.channel_count as usize,
                    ),
                    frames_count: self.frames_count,
                })
            } else {
                SampleType::F64(TAudioChannelsMut {
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

    #[inline]
    pub fn constant_mask_mut(&mut self) -> &mut ConstantMask {
        ConstantMask::from_bits_mut(&mut self.inner.constant_mask)
    }
}
