use clap_sys::audio_buffer::clap_audio_buffer;
use std::ops::DerefMut;

pub struct ChannelBuffer<T> {
    pub data: T,
    pub is_constant: bool,
}

impl<T> ChannelBuffer<T> {
    #[inline]
    pub fn variable(data: T) -> Self {
        Self {
            data,
            is_constant: false,
        }
    }
}

pub enum AudioPortBufferType<I32, I64> {
    F32(I32),
    F64(I64),
}

pub struct AudioPortBuffer<I32, I64> {
    pub channels: AudioPortBufferType<I32, I64>,
    pub latency: u32,
}

// bikeshed
pub struct AudioPorts {
    buffer_lists: Vec<*mut f32>, // Can be f32 or f64, casted on-demand
    buffer_configs: Vec<clap_audio_buffer>,
}

// SAFETY: The pointers are only temporary storage, they are not used unless AudioPorts is exclusively borrowed
unsafe impl Send for AudioPorts {}
unsafe impl Sync for AudioPorts {}

impl AudioPorts {
    pub fn with_capacity(channel_count: usize, port_count: usize) -> Self {
        let mut bufs = Self {
            buffer_configs: Vec::with_capacity(port_count),
            buffer_lists: Vec::with_capacity(port_count * channel_count),
        };
        bufs.resize_buffer_configs(port_count);

        bufs
    }

    #[inline]
    pub fn port_capacity(&self) -> usize {
        self.buffer_configs.len()
    }

    fn resize_buffer_configs(&mut self, new_size: usize) {
        if new_size > self.buffer_configs.len() {
            self.buffer_configs.resize(
                new_size,
                clap_audio_buffer {
                    data32: core::ptr::null_mut(),
                    data64: core::ptr::null_mut(),
                    channel_count: 0,
                    latency: 0,
                    constant_mask: 0,
                },
            );
        }
    }

    pub fn with_data<'a, I, Iter, ChannelIter32, ChannelIter64, TBuf32, TBuf64>(
        &'a mut self,
        iter: I,
    ) -> AudioBuffers<'a>
    where
        I: IntoIterator<Item = AudioPortBuffer<ChannelIter32, ChannelIter64>, IntoIter = Iter>,
        Iter: ExactSizeIterator<Item = AudioPortBuffer<ChannelIter32, ChannelIter64>>,
        ChannelIter32: Iterator<Item = ChannelBuffer<TBuf32>>,
        ChannelIter64: Iterator<Item = ChannelBuffer<TBuf64>>,
        TBuf32: DerefMut<Target = [f32]> + 'a,
        TBuf64: DerefMut<Target = [f64]> + 'a,
    {
        let iter = iter.into_iter();
        self.resize_buffer_configs(iter.len());
        self.buffer_lists.clear();

        let mut min_buffer_length = usize::MAX;
        let mut total = 0;

        for (i, port) in iter.enumerate() {
            total = i + 1;

            let last = self.buffer_lists.len();

            let mut constant_mask = 0u64;
            let is_f64 = match port.channels {
                AudioPortBufferType::F32(channels) => {
                    for mut channel in channels {
                        min_buffer_length = min_buffer_length.min(channel.data.len());
                        if channel.is_constant {
                            constant_mask |= 1 << i as u64
                        }

                        self.buffer_lists.push(channel.data.as_mut_ptr().cast())
                    }
                    false
                }
                AudioPortBufferType::F64(channels) => {
                    for mut channel in channels {
                        min_buffer_length = min_buffer_length.min(channel.data.len());
                        if channel.is_constant {
                            constant_mask |= 1 << i as u64
                        }

                        self.buffer_lists.push(channel.data.as_mut_ptr().cast())
                    }
                    true
                }
            };

            let buffers = self.buffer_lists.get_mut(last..).unwrap_or(&mut []);

            // PANIC: this can only panic with an invalid implementation of ExactSizeIterator
            let descriptor = &mut self.buffer_configs[i];
            descriptor.channel_count = buffers.len() as u32;
            descriptor.latency = port.latency;
            descriptor.constant_mask = constant_mask;

            if is_f64 {
                descriptor.data64 = buffers.as_ptr().cast();
                descriptor.data32 = core::ptr::null();
            } else {
                descriptor.data64 = core::ptr::null();
                descriptor.data32 = buffers.as_ptr() as *const *const _;
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
