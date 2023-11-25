use clap_sys::audio_buffer::clap_audio_buffer;
use core::array::IntoIter;

mod ports_info;
pub use ports_info::*;

pub struct InputChannel<'a, T> {
    pub buffer: &'a mut [T],
    pub is_constant: bool,
}

impl<'a, T> InputChannel<'a, T> {
    #[inline]
    pub fn from_buffer<D: ?Sized + AsMut<[T]> + 'a>(buffer: &'a mut D, is_constant: bool) -> Self {
        Self {
            buffer: buffer.as_mut(),
            is_constant,
        }
    }

    #[inline]
    pub fn variable<D: ?Sized + AsMut<[T]> + 'a>(buffer: &'a mut D) -> Self {
        Self::from_buffer(buffer, false)
    }

    #[inline]
    pub fn constant<D: ?Sized + AsMut<[T]> + 'a>(buffer: &'a mut D) -> Self {
        Self::from_buffer(buffer, true)
    }
}

pub enum AudioPortBufferType<I32, I64> {
    F32(I32),
    F64(I64),
}

impl<I32> AudioPortBufferType<I32, IntoIter<InputChannel<'static, f64>, 0>> {
    #[inline]
    pub fn f32_input_only(iterator: I32) -> Self {
        Self::F32(iterator)
    }
}

impl<I32> AudioPortBufferType<I32, IntoIter<&'static mut [f64], 0>> {
    #[inline]
    pub fn f32_output_only(iterator: I32) -> Self {
        Self::F32(iterator)
    }
}

impl<I64> AudioPortBufferType<IntoIter<InputChannel<'static, f32>, 0>, I64> {
    #[inline]
    pub fn f64_input_only(iterator: I64) -> Self {
        Self::F64(iterator)
    }
}

impl<I64> AudioPortBufferType<IntoIter<&'static mut [f32], 0>, I64> {
    #[inline]
    pub fn f64_output_only(iterator: I64) -> Self {
        Self::F64(iterator)
    }
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
    pub fn with_capacity(total_channel_count: usize, port_count: usize) -> Self {
        let mut bufs = Self {
            buffer_configs: Vec::with_capacity(port_count),
            buffer_lists: Vec::with_capacity(total_channel_count),
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

    pub fn with_input_buffers<'a, I, Iter, ChannelIter32, ChannelIter64>(
        &'a mut self,
        iter: I,
    ) -> InputAudioBuffers<'a>
    where
        I: IntoIterator<Item = AudioPortBuffer<ChannelIter32, ChannelIter64>, IntoIter = Iter>,
        Iter: ExactSizeIterator<Item = AudioPortBuffer<ChannelIter32, ChannelIter64>>,
        ChannelIter32: IntoIterator<Item = InputChannel<'a, f32>>,
        ChannelIter64: IntoIterator<Item = InputChannel<'a, f64>>,
    {
        let iter = iter.into_iter();
        self.resize_buffer_configs(iter.len());
        self.buffer_lists.clear();

        let mut min_channel_buffer_length = usize::MAX;
        let mut total = 0;
        let mut has_reallocated = false;

        for (i, port) in iter.enumerate() {
            total = i + 1;

            let last = self.buffer_lists.len();

            let mut constant_mask = 0u64;
            let is_f64 = match port.channels {
                AudioPortBufferType::F32(channels) => {
                    for channel in channels {
                        min_channel_buffer_length =
                            min_channel_buffer_length.min(channel.buffer.len());
                        if channel.is_constant {
                            constant_mask |= 1 << i as u64
                        }

                        if self.buffer_lists.len() >= self.buffer_lists.capacity() {
                            has_reallocated = true;
                        }

                        self.buffer_lists.push(channel.buffer.as_mut_ptr().cast())
                    }
                    false
                }
                AudioPortBufferType::F64(channels) => {
                    for channel in channels {
                        min_channel_buffer_length =
                            min_channel_buffer_length.min(channel.buffer.len());
                        if channel.is_constant {
                            constant_mask |= 1 << i as u64
                        }

                        if self.buffer_lists.len() >= self.buffer_lists.capacity() {
                            has_reallocated = true;
                        }

                        self.buffer_lists.push(channel.buffer.as_mut_ptr().cast())
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

        // If a realloc occured, we must rewrite all of the pointers.
        // Thankfully, we know we wrote them sequentially, and we stored the lengths, so it's easy
        // to find them back.
        if has_reallocated {
            let mut last_len = 0;
            for descriptor in &mut self.buffer_configs[..total] {
                let channel_count = descriptor.channel_count as usize;
                let buffers = self
                    .buffer_lists
                    .get_mut(last_len..channel_count)
                    .unwrap_or(&mut []);
                last_len += channel_count;

                if descriptor.data32.is_null() {
                    descriptor.data64 = buffers.as_ptr().cast();
                } else {
                    descriptor.data32 = buffers.as_ptr().cast();
                }
            }
        }

        InputAudioBuffers {
            buffers: &self.buffer_configs[..total],
            min_channel_buffer_length,
        }
    }

    pub fn with_output_buffers<'a, I, Iter, ChannelIter32, ChannelIter64>(
        &'a mut self,
        iter: I,
    ) -> OutputAudioBuffers<'a>
    where
        I: IntoIterator<Item = AudioPortBuffer<ChannelIter32, ChannelIter64>, IntoIter = Iter>,
        Iter: ExactSizeIterator<Item = AudioPortBuffer<ChannelIter32, ChannelIter64>>,
        ChannelIter32: IntoIterator<Item = &'a mut [f32]>,
        ChannelIter64: IntoIterator<Item = &'a mut [f64]>,
    {
        let iter = iter.into_iter();
        self.resize_buffer_configs(iter.len());
        self.buffer_lists.clear();

        let mut min_channel_buffer_length = usize::MAX;
        let mut total = 0;
        let mut has_reallocated = false;

        for (i, port) in iter.enumerate() {
            total = i + 1;

            let last = self.buffer_lists.len();

            let is_f64 = match port.channels {
                AudioPortBufferType::F32(channels) => {
                    for channel in channels {
                        min_channel_buffer_length = min_channel_buffer_length.min(channel.len());

                        if self.buffer_lists.len() >= self.buffer_lists.capacity() {
                            has_reallocated = true;
                        }

                        self.buffer_lists.push(channel.as_mut_ptr().cast())
                    }
                    false
                }
                AudioPortBufferType::F64(channels) => {
                    for channel in channels {
                        min_channel_buffer_length = min_channel_buffer_length.min(channel.len());

                        if self.buffer_lists.len() >= self.buffer_lists.capacity() {
                            has_reallocated = true;
                        }

                        self.buffer_lists.push(channel.as_mut_ptr().cast())
                    }
                    true
                }
            };

            let buffers = self.buffer_lists.get_mut(last..).unwrap_or(&mut []);

            // PANIC: this can only panic with an invalid implementation of ExactSizeIterator
            let descriptor = &mut self.buffer_configs[i];
            descriptor.channel_count = buffers.len() as u32;
            descriptor.latency = port.latency;
            descriptor.constant_mask = 0;

            if is_f64 {
                descriptor.data64 = buffers.as_mut_ptr().cast();
                descriptor.data32 = core::ptr::null();
            } else {
                descriptor.data64 = core::ptr::null();
                descriptor.data32 = buffers.as_mut_ptr() as *const *const _;
            }
        }

        // If a realloc occured, we must rewrite all of the pointers.
        // Thankfully, we know we wrote them sequentially, and we stored the lengths, so it's easy
        // to find them back.
        if has_reallocated {
            let mut last_len = 0;
            for descriptor in &mut self.buffer_configs[..total] {
                let channel_count = descriptor.channel_count as usize;
                let buffers = self
                    .buffer_lists
                    .get_mut(last_len..channel_count)
                    .unwrap_or(&mut []);
                last_len += channel_count;

                if descriptor.data32.is_null() {
                    descriptor.data64 = buffers.as_ptr().cast();
                } else {
                    descriptor.data32 = buffers.as_ptr().cast();
                }
            }
        }

        OutputAudioBuffers {
            buffers: &mut self.buffer_configs[..total],
            min_channel_buffer_length,
        }
    }

    #[inline]
    pub fn port_info(&self, port_index: u32) -> Option<OutputAudioPortInfo> {
        self.buffer_configs
            .get(port_index as usize)
            .map(|buffer| OutputAudioPortInfo { buffer })
    }

    #[inline]
    pub fn port_count(&self) -> usize {
        self.buffer_configs.len()
    }

    #[inline]
    pub fn port_infos(&self) -> impl Iterator<Item = OutputAudioPortInfo> {
        self.buffer_configs
            .iter()
            .map(|buffer| OutputAudioPortInfo { buffer })
    }
}

pub struct InputAudioBuffers<'a> {
    buffers: &'a [clap_audio_buffer],
    min_channel_buffer_length: usize,
}

impl<'a> InputAudioBuffers<'a> {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            buffers: &[],
            min_channel_buffer_length: 0,
        }
    }

    #[inline]
    pub fn as_raw_buffers(&self) -> &'a [clap_audio_buffer] {
        self.buffers
    }

    #[inline]
    pub fn min_channel_buffer_length(&self) -> usize {
        self.min_channel_buffer_length
    }
}

pub struct OutputAudioBuffers<'a> {
    buffers: &'a mut [clap_audio_buffer],
    min_channel_buffer_length: usize,
}

impl<'a> OutputAudioBuffers<'a> {
    #[inline]
    pub fn empty() -> Self {
        Self {
            buffers: &mut [],
            min_channel_buffer_length: 0,
        }
    }

    #[inline]
    pub fn as_raw_buffers(&mut self) -> &mut [clap_audio_buffer] {
        self.buffers
    }

    #[inline]
    pub fn into_raw_buffers(self) -> &'a mut [clap_audio_buffer] {
        self.buffers
    }

    #[inline]
    pub fn min_channel_buffer_length(&self) -> usize {
        self.min_channel_buffer_length
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use clack_plugin::prelude::*;
    use clap_sys::process::clap_process;
    use std::cell::RefCell;
    use std::ptr::null_mut;

    #[test]
    pub fn input_audio_buffers_work() {
        let mut ports = AudioPorts::with_capacity(2, 1);
        let mut bufs = [[0f32; 4]; 2];

        let buffers = ports.with_input_buffers([AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_input_only(bufs.iter_mut().map(|b| InputChannel {
                buffer: b.as_mut_slice(),
                is_constant: false,
            })),
        }]);

        assert_eq!(buffers.buffers.len(), 1);
        assert_eq!(buffers.min_channel_buffer_length, 4);
    }

    #[test]
    pub fn output_audio_buffers_work() {
        let mut ports = AudioPorts::with_capacity(2, 1);
        let mut bufs = [[0f32; 4]; 2];

        let buffers = ports.with_output_buffers([AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_output_only(
                bufs.iter_mut().map(|b| b.as_mut_slice()),
            ),
        }]);

        assert_eq!(buffers.buffers.len(), 1);
        assert_eq!(buffers.min_channel_buffer_length, 4);

        assert_eq!(bufs.len(), 2); // Check borrow still works
        assert_eq!(ports.port_count(), 1);
    }

    #[test]
    pub fn input_audio_buffers_work_with_refcell() {
        let mut ports = AudioPorts::with_capacity(2, 1);
        let bufs = [RefCell::new([0f32; 4]), RefCell::new([0f32; 4])];
        let mut borrowed: Vec<_> = bufs.iter().map(|c| c.borrow_mut()).collect();

        let buffers = ports.with_input_buffers([AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_input_only(borrowed.iter_mut().map(|b| {
                InputChannel {
                    buffer: b.as_mut_slice(),
                    is_constant: false,
                }
            })),
        }]);

        assert_eq!(buffers.buffers.len(), 1);
        assert_eq!(buffers.min_channel_buffer_length, 4);
    }

    #[test]
    pub fn output_audio_buffers_work_with_refcell() {
        let mut ports = AudioPorts::with_capacity(2, 1);
        let bufs = [RefCell::new([0f32; 4]), RefCell::new([0f32; 4])];
        let mut borrowed: Vec<_> = bufs.iter().map(|c| c.borrow_mut()).collect();

        let buffers = ports.with_output_buffers([AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_output_only(
                borrowed.iter_mut().map(|b| b.as_mut_slice()),
            ),
        }]);

        assert_eq!(buffers.buffers.len(), 1);
        assert_eq!(buffers.min_channel_buffer_length, 4);

        assert_eq!(bufs.len(), 2); // Check borrow still works
        assert_eq!(ports.port_count(), 1);
    }

    #[test]
    pub fn audio_buffers_work_with_wrong_capacity() {
        let mut input_ports = AudioPorts::with_capacity(1, 1);
        let mut output_ports = AudioPorts::with_capacity(1, 1);
        let mut input_bufs = [[[42f32; 4]; 128], [[69f32; 4]; 128]];
        let mut output_bufs = [[[42f32; 4]; 128], [[69f32; 4]; 128]];

        let input_buffers =
            input_ports.with_input_buffers(input_bufs.iter_mut().map(|bufs| AudioPortBuffer {
                latency: 0,
                channels: AudioPortBufferType::f32_input_only(bufs.iter_mut().map(|b| {
                    InputChannel {
                        buffer: b.as_mut_slice(),
                        is_constant: false,
                    }
                })),
            }));

        let mut output_buffers =
            output_ports.with_output_buffers(output_bufs.iter_mut().map(|bufs| AudioPortBuffer {
                latency: 0,
                channels: AudioPortBufferType::f32_output_only(
                    bufs.iter_mut().map(|b| b.as_mut_slice()),
                ),
            }));

        assert_eq!(input_buffers.buffers.len(), 2);
        assert_eq!(input_buffers.min_channel_buffer_length, 4);
        assert_eq!(output_buffers.buffers.len(), 2);
        assert_eq!(output_buffers.min_channel_buffer_length, 4);

        let raw_input_buffers = input_buffers.as_raw_buffers();
        let raw_output_buffers = output_buffers.as_raw_buffers();
        let process = clap_process {
            audio_inputs: raw_input_buffers.as_ptr(),
            audio_outputs: raw_output_buffers.as_ptr() as *mut _,
            audio_inputs_count: raw_input_buffers.len() as u32,
            audio_outputs_count: raw_output_buffers.len() as u32,

            steady_time: 0,
            frames_count: 4,
            transport: null_mut(),
            in_events: null_mut(),
            out_events: null_mut(),
        };

        let mut audio = unsafe { Audio::from_raw(&process) };

        for (port, bufs) in audio.input_ports().zip(&input_bufs) {
            let channels = port.channels().unwrap().into_f32().unwrap();

            assert_eq!(channels.channel_count(), 128);
            for (channel, buf) in channels.iter().zip(bufs.iter()) {
                assert_eq!(buf, channel)
            }
        }

        for (mut port, bufs) in audio.output_ports().zip(&output_bufs) {
            let channels = port.channels().unwrap().into_f32().unwrap();

            assert_eq!(channels.channel_count(), 128);
            for (channel, buf) in channels.iter().zip(bufs.iter()) {
                assert_eq!(buf, channel)
            }
        }
    }
}
