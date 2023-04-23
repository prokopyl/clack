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
    pub fn map_option<TF32, TF64, Fn32, Fn64>(
        self,
        fn32: Fn32,
        fn64: Fn64,
    ) -> Option<SampleType<TF32, TF64>>
    where
        Fn32: FnOnce(F32) -> Option<TF32>,
        Fn64: FnOnce(F64) -> Option<TF64>,
    {
        match self {
            SampleType::F32(c) => Some(SampleType::F32(fn32(c)?)),
            SampleType::F64(c) => Some(SampleType::F64(fn64(c)?)),
            SampleType::Both(c32, c64) => match (fn32(c32), fn64(c64)) {
                (Some(c32), Some(c64)) => Some(SampleType::Both(c32, c64)),
                (Some(c), None) => Some(SampleType::F32(c)),
                (None, Some(c)) => Some(SampleType::F64(c)),
                (None, None) => None,
            },
        }
    }

    #[inline]
    pub fn try_match_with<TF32, TF64>(
        self,
        other: SampleType<TF32, TF64>,
    ) -> Option<SampleType<(F32, TF32), (F64, TF64)>> {
        match (self, other) {
            (
                SampleType::F32(s) | SampleType::Both(s, _),
                SampleType::F32(o) | SampleType::Both(o, _),
            ) => Some(SampleType::F32((s, o))),
            (
                SampleType::F64(s) | SampleType::Both(_, s),
                SampleType::F64(o) | SampleType::Both(_, o),
            ) => Some(SampleType::F64((s, o))),
            _ => None,
        }
    }
}

impl<'a> SampleType<&'a [*const f32], &'a [*const f64]> {
    #[inline]
    pub(crate) unsafe fn from_raw_buffer(raw: &clap_audio_buffer) -> Option<Self> {
        match (raw.data32.is_null(), raw.data64.is_null()) {
            (true, true) => None,
            (false, true) => Some(SampleType::F32(core::slice::from_raw_parts(
                raw.data32,
                raw.channel_count as usize,
            ))),
            (true, false) => Some(SampleType::F64(core::slice::from_raw_parts(
                raw.data64,
                raw.channel_count as usize,
            ))),
            (false, false) => Some(SampleType::Both(
                core::slice::from_raw_parts(raw.data32, raw.channel_count as usize),
                core::slice::from_raw_parts(raw.data64, raw.channel_count as usize),
            )),
        }
    }
}

impl<'a> SampleType<&'a mut [*const f32], &'a mut [*const f64]> {
    #[inline]
    pub(crate) unsafe fn from_raw_buffer_mut(raw: &mut clap_audio_buffer) -> Option<Self> {
        match (raw.data32.is_null(), raw.data64.is_null()) {
            (true, true) => None,
            (false, true) => Some(SampleType::F32(core::slice::from_raw_parts_mut(
                raw.data32 as *mut _,
                raw.channel_count as usize,
            ))),
            (true, false) => Some(SampleType::F64(core::slice::from_raw_parts_mut(
                raw.data64 as *mut _,
                raw.channel_count as usize,
            ))),
            (false, false) => Some(SampleType::Both(
                core::slice::from_raw_parts_mut(raw.data32 as *mut _, raw.channel_count as usize),
                core::slice::from_raw_parts_mut(raw.data64 as *mut _, raw.channel_count as usize),
            )),
        }
    }
}
