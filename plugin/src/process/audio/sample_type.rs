use crate::process::audio::BufferError;
use clap_sys::audio_buffer::clap_audio_buffer;

pub enum SampleType<F32, F64> {
    F32(F32),
    F64(F64),
    Both(F32, F64),
}

impl<F32, F64> SampleType<F32, F64> {
    #[inline]
    pub fn as_f32(&self) -> Option<&F32> {
        match self {
            SampleType::F32(c) | SampleType::Both(c, _) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f32_mut(&mut self) -> Option<&mut F32> {
        match self {
            SampleType::F32(c) | SampleType::Both(c, _) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn into_f32(self) -> Option<F32> {
        match self {
            SampleType::F32(c) | SampleType::Both(c, _) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f64(&self) -> Option<&F64> {
        match self {
            SampleType::F64(c) | SampleType::Both(_, c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f64_mut(&mut self) -> Option<&mut F64> {
        match self {
            SampleType::F64(c) | SampleType::Both(_, c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn into_f64(self) -> Option<F64> {
        match self {
            SampleType::F64(c) | SampleType::Both(_, c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn map<TF32, TF64, Fn32, Fn64>(self, fn32: Fn32, fn64: Fn64) -> SampleType<TF32, TF64>
    where
        Fn32: FnOnce(F32) -> TF32,
        Fn64: FnOnce(F64) -> TF64,
    {
        match self {
            SampleType::F32(c) => SampleType::F32(fn32(c)),
            SampleType::F64(c) => SampleType::F64(fn64(c)),
            SampleType::Both(c32, c64) => SampleType::Both(fn32(c32), fn64(c64)),
        }
    }

    #[inline]
    #[allow(clippy::type_complexity)] // I'm sorry
    pub fn try_match_with<TF32, TF64>(
        self,
        other: SampleType<TF32, TF64>,
    ) -> Result<SampleType<(F32, TF32), (F64, TF64)>, BufferError> {
        match (self, other) {
            (
                SampleType::F32(s) | SampleType::Both(s, _),
                SampleType::F32(o) | SampleType::Both(o, _),
            ) => Ok(SampleType::F32((s, o))),
            (
                SampleType::F64(s) | SampleType::Both(_, s),
                SampleType::F64(o) | SampleType::Both(_, o),
            ) => Ok(SampleType::F64((s, o))),
            _ => Err(BufferError::MismatchedBufferPair),
        }
    }
}

impl<'a> SampleType<&'a [*const f32], &'a [*const f64]> {
    #[inline]
    pub(crate) unsafe fn from_raw_buffer(raw: &clap_audio_buffer) -> Result<Self, BufferError> {
        match (raw.data32.is_null(), raw.data64.is_null()) {
            (true, true) => {
                if raw.channel_count == 0 {
                    Ok(SampleType::Both([].as_slice(), [].as_slice()))
                } else {
                    Err(BufferError::InvalidChannelBuffer)
                }
            }
            (false, true) => Ok(SampleType::F32(core::slice::from_raw_parts(
                raw.data32,
                raw.channel_count as usize,
            ))),
            (true, false) => Ok(SampleType::F64(core::slice::from_raw_parts(
                raw.data64,
                raw.channel_count as usize,
            ))),
            (false, false) => Ok(SampleType::Both(
                core::slice::from_raw_parts(raw.data32, raw.channel_count as usize),
                core::slice::from_raw_parts(raw.data64, raw.channel_count as usize),
            )),
        }
    }
}

impl<'a> SampleType<&'a mut [*const f32], &'a mut [*const f64]> {
    #[inline]
    pub(crate) unsafe fn from_raw_buffer_mut(
        raw: &mut clap_audio_buffer,
    ) -> Result<Self, BufferError> {
        match (raw.data32.is_null(), raw.data64.is_null()) {
            (true, true) => {
                if raw.channel_count == 0 {
                    Ok(SampleType::Both([].as_mut_slice(), [].as_mut_slice()))
                } else {
                    Err(BufferError::InvalidChannelBuffer)
                }
            }
            (false, true) => Ok(SampleType::F32(core::slice::from_raw_parts_mut(
                raw.data32 as *mut _,
                raw.channel_count as usize,
            ))),
            (true, false) => Ok(SampleType::F64(core::slice::from_raw_parts_mut(
                raw.data64 as *mut _,
                raw.channel_count as usize,
            ))),
            (false, false) => Ok(SampleType::Both(
                core::slice::from_raw_parts_mut(raw.data32 as *mut _, raw.channel_count as usize),
                core::slice::from_raw_parts_mut(raw.data64 as *mut _, raw.channel_count as usize),
            )),
        }
    }
}
