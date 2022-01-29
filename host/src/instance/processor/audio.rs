use clap_sys::audio_buffer::clap_audio_buffer;

pub struct ChannelBuffer<'a, S> {
    pub data: &'a mut [S],
    pub is_constant: bool,
}

impl<'a, S> ChannelBuffer<'a, S> {
    #[inline]
    pub fn variable(data: &'a mut [S]) -> Self {
        Self {
            data,
            is_constant: false,
        }
    }
}

pub struct AudioBuffer<I> {
    pub channels: I,
    pub latency: u32,
}

// bikeshed
pub struct AudioPorts {
    buffer_lists: Vec<*mut f32>, // Can be f32 or f64, casted on-demand
    buffer_configs: Vec<clap_audio_buffer>,
}

impl AudioPorts {
    pub fn with_capacity(channel_count: usize, port_count: usize) -> Self {
        let mut bufs = Self {
            buffer_configs: Vec::with_capacity(port_count),
            buffer_lists: Vec::with_capacity(port_count * channel_count),
        };
        bufs.resize_buffer_configs(port_count);

        bufs
    }

    fn resize_buffer_configs(&mut self, new_size: usize) {
        if new_size > self.buffer_configs.len() {
            self.buffer_configs.resize(
                new_size,
                clap_audio_buffer {
                    data32: ::core::ptr::null_mut(),
                    data64: ::core::ptr::null_mut(),
                    channel_count: 0,
                    latency: 0,
                    constant_mask: 0,
                },
            );
        }
    }

    pub fn with_buffers_f32<'a, I, Iter, ChannelIter>(&'a mut self, iter: I) -> AudioBuffers<'a>
    where
        I: IntoIterator<Item = AudioBuffer<ChannelIter>, IntoIter = Iter>,
        Iter: ExactSizeIterator<Item = AudioBuffer<ChannelIter>>,
        ChannelIter: Iterator<Item = ChannelBuffer<'a, f32>>,
    {
        // SAFETY: pointer is guaranteed to be f32
        unsafe { self.with_data(iter.into_iter(), false) }
    }

    pub fn with_buffers_f64<'a, I, Iter, ChannelIter>(&'a mut self, iter: I) -> AudioBuffers<'a>
    where
        I: IntoIterator<Item = AudioBuffer<ChannelIter>, IntoIter = Iter>,
        Iter: ExactSizeIterator<Item = AudioBuffer<ChannelIter>>,
        ChannelIter: Iterator<Item = ChannelBuffer<'a, f64>>,
    {
        // SAFETY: pointer is guaranteed to be f64
        unsafe { self.with_data(iter.into_iter(), true) }
    }

    /// # Safety
    /// Caller must ensure the sample type S is correctly either f32 or f64
    unsafe fn with_data<'a, I, S: 'static, ChannelIter>(
        &'a mut self,
        iter: I,
        is_f64: bool,
    ) -> AudioBuffers<'a>
    where
        I: ExactSizeIterator<Item = AudioBuffer<ChannelIter>>,
        ChannelIter: Iterator<Item = ChannelBuffer<'a, S>>,
    {
        self.resize_buffer_configs(iter.len());
        self.buffer_lists.clear();

        let mut min_buffer_length = usize::MAX;
        let mut total = 0;

        for (i, port) in iter.enumerate() {
            total = i + 1;

            let last = self.buffer_lists.len();

            let mut constant_mask = 0u64;
            for channel in port.channels {
                min_buffer_length = min_buffer_length.min(channel.data.len());
                if channel.is_constant {
                    constant_mask |= 1 << i as u64
                }

                self.buffer_lists.push(channel.data.as_mut_ptr().cast())
            }

            let buffers = self.buffer_lists.get_mut(last..).unwrap_or(&mut []);

            // PANIC: this can only panic with an invalid implementation of ExactSizeIterator
            let descriptor = &mut self.buffer_configs[i];
            descriptor.channel_count = buffers.len() as u32;
            descriptor.latency = port.latency;
            descriptor.constant_mask = constant_mask;

            if is_f64 {
                descriptor.data64 = buffers.as_mut_ptr().cast();
                descriptor.data32 = ::core::ptr::null_mut();
            } else {
                descriptor.data64 = ::core::ptr::null_mut();
                descriptor.data32 = buffers.as_mut_ptr();
            }
        }

        AudioBuffers {
            buffers: &mut self.buffer_configs[..total],
            min_buffer_length,
        }
    }
}

pub struct AudioBuffers<'a> {
    pub(crate) buffers: &'a mut [clap_audio_buffer],
    pub(crate) min_buffer_length: usize,
}
