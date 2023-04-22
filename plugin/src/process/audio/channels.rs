use clap_sys::audio_buffer::clap_audio_buffer;

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
            ChannelPair::InPlace(output)
        } else {
            ChannelPair::InputOutput(input, output)
        }
    }
}

pub enum SampleType<F32, F64> {
    F32(F32),
    F64(F64),
    Both(F32, F64),
}

// TODO: Implement Both
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

// TODO: bikeshed
#[derive(Copy, Clone)]
pub struct TAudioChannels<'a, S> {
    pub(crate) frames_count: u32,
    pub(crate) data: &'a [*const S],
}

impl<'a, S> TAudioChannels<'a, S> {
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
    pub fn iter(&self) -> TAudioChannelsIter<S> {
        TAudioChannelsIter {
            data: self.data.iter(),
            frames_count: self.frames_count,
        }
    }
}

impl<'a, T> IntoIterator for &'a TAudioChannels<'a, T> {
    type Item = &'a [T];
    type IntoIter = TAudioChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// TODO: bikeshed
pub struct TAudioChannelsMut<'a, S> {
    pub(crate) frames_count: u32,
    pub(crate) data: &'a [*const S],
}

impl<'a, S> TAudioChannelsMut<'a, S> {
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.data.len() as u32
    }

    #[inline]
    pub fn get_channel_data(&self, channel_index: usize) -> Option<&'a [S]> {
        unsafe {
            self.data.get(channel_index).map(|data| {
                core::slice::from_raw_parts(*data as *const _, self.frames_count as usize)
            })
        }
    }

    #[inline]
    pub fn get_channel_data_mut(&mut self, channel_index: usize) -> Option<&'a mut [S]> {
        unsafe {
            self.data.get(channel_index).map(|data| {
                core::slice::from_raw_parts_mut(*data as *mut _, self.frames_count as usize)
            })
        }
    }

    #[inline]
    pub fn iter(&self) -> TAudioChannelsIter<S> {
        TAudioChannelsIter {
            data: self.data.iter(),
            frames_count: self.frames_count,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> TAudioChannelsIterMut<S> {
        TAudioChannelsIterMut {
            data: self.data.as_ref().iter(),
            frames_count: self.frames_count,
        }
    }
}

pub struct TAudioChannelsIter<'a, T> {
    data: core::slice::Iter<'a, *const T>,
    frames_count: u32,
}

impl<'a, T> Iterator for TAudioChannelsIter<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.data
            .next()
            .map(|ptr| unsafe { core::slice::from_raw_parts(*ptr, self.frames_count as usize) })
    }
}

impl<'a, T> IntoIterator for &'a TAudioChannelsMut<'a, T> {
    type Item = &'a [T];
    type IntoIter = TAudioChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut TAudioChannelsMut<'a, T> {
    type Item = &'a mut [T];
    type IntoIter = TAudioChannelsIterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct TAudioChannelsIterMut<'a, T> {
    data: core::slice::Iter<'a, *const T>,
    frames_count: u32,
}

impl<'a, T> Iterator for TAudioChannelsIterMut<'a, T> {
    type Item = &'a mut [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|ptr| unsafe {
            core::slice::from_raw_parts_mut(*ptr as *mut _, self.frames_count as usize)
        })
    }
}
