pub enum AudioBufferType<F32, F64> {
    F32(F32),
    F64(F64),
}

impl<F32, F64> AudioBufferType<F32, F64> {
    #[inline]
    pub fn as_f32(&self) -> Option<&F32> {
        match self {
            AudioBufferType::F32(c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f32_mut(&mut self) -> Option<&mut F32> {
        match self {
            AudioBufferType::F32(c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn into_f32(self) -> Option<F32> {
        match self {
            AudioBufferType::F32(c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f64(&self) -> Option<&F64> {
        match self {
            AudioBufferType::F64(c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn as_f64_mut(&mut self) -> Option<&mut F64> {
        match self {
            AudioBufferType::F64(c) => Some(c),
            _ => None,
        }
    }

    #[inline]
    pub fn into_f64(self) -> Option<F64> {
        match self {
            AudioBufferType::F64(c) => Some(c),
            _ => None,
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

    #[inline]
    pub fn iter_mut(&mut self) -> TAudioChannelsIterMut<S> {
        TAudioChannelsIterMut {
            data: self.data.as_ref().iter(),
            frames_count: self.frames_count,
        }
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
