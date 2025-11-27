//! Types to manipulate input and output audio buffers for processing.

use crate::process::celled_audio_buffers::CelledClapAudioBuffer;
use crate::process::into_buffers::{ChannelBuffer, IntoPortsBuffers, PortBuffers};
use clack_common::process::{AudioPortProcessingInfo, ConstantMask};
use clap_sys::audio_buffer::clap_audio_buffer;
use core::array::IntoIter;

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

impl<'a> ChannelBuffer<'a> for InputChannel<'a, f32> {
    const IS_F64: bool = false;

    #[inline]
    fn buf_len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    fn is_constant(&self) -> bool {
        self.is_constant
    }

    #[inline]
    fn buf_ptr(&mut self) -> *mut f32 {
        self.buffer.as_mut_ptr()
    }
}

pub enum AudioPortBufferType<I32, I64> {
    F32(I32),
    F64(I64),
}

pub trait IntoChannelsBuffers<'a> {
    type ChannelBuffer: ChannelBuffer<'a>;
    fn into_channels_buffers(self) -> impl Iterator<Item = Self::ChannelBuffer>;
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

impl<F32, F64> PortBuffers<'_> for AudioPortBuffer<F32, F64> {
    type ChannelBuffer = ();

    fn is_f64(&self) -> bool {
        todo!()
    }

    fn latency(&self) -> u32 {
        todo!()
    }

    fn into_channels_refs(self) -> impl Iterator<Item = Self::ChannelBuffer> {
        todo!();
        core::iter::empty()
    }
}

// TODO: bikeshed
pub struct AudioPorts {
    buffer_lists: Vec<*mut f32>, // Can be f32 or f64, cast on-demand
    buffer_configs: Vec<clap_audio_buffer>,
}

// SAFETY: The pointers are only temporary storage, they are not used unless AudioPorts is exclusively borrowed
unsafe impl Send for AudioPorts {}
// SAFETY: The pointers are only temporary storage, they are not used unless AudioPorts is exclusively borrowed
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

    pub fn with_buffers<'a>(&'a mut self, bufs: impl IntoPortsBuffers<'a>) -> AudioBuffers<'a> {
        let port_count = bufs.port_count();
        self.resize_buffer_configs(port_count);
        self.buffer_lists.clear();

        let mut min_channel_buffer_length = usize::MAX;
        let mut total = 0;
        let mut has_reallocated = false;

        for (i, port) in bufs.into_iterator().enumerate() {
            total = i + 1;

            let latency = port.latency();

            let last = self.buffer_lists.len();

            let mut constant_mask = ConstantMask::FULLY_DYNAMIC;
            let is_f64 = port.is_f64();

            for mut channel in port.into_channels_refs() {
                min_channel_buffer_length = min_channel_buffer_length.min(channel.buf_len());
                constant_mask.set_channel_constant(i as u64, channel.is_constant());

                if self.buffer_lists.len() >= self.buffer_lists.capacity() {
                    has_reallocated = true;
                }

                self.buffer_lists.push(channel.buf_ptr())
            }

            let buffers = self.buffer_lists.get_mut(last..).unwrap_or(&mut []);

            // PANIC: this can only panic with an invalid implementation of ExactSizeIterator
            let descriptor = &mut self.buffer_configs[i];
            descriptor.channel_count = buffers.len() as u32;
            descriptor.latency = latency;
            descriptor.constant_mask = constant_mask.to_bits();

            if is_f64 {
                descriptor.data64 = buffers.as_ptr().cast();
                descriptor.data32 = core::ptr::null();
            } else {
                descriptor.data64 = core::ptr::null();
                descriptor.data32 = buffers.as_ptr() as *const *const _;
            }
        }

        // If a realloc occurred, we must rewrite all the pointers.
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

        AudioBuffers {
            buffers: CelledClapAudioBuffer::from_raw_slice(&mut self.buffer_configs[..total]),
            frames_count: if min_channel_buffer_length == usize::MAX {
                None
            } else {
                Some(min_channel_buffer_length as u32)
            },
        }
    }

    pub fn with_output_buffers<'a, I, Iter, ChannelIter32, ChannelIter64>(
        &'a mut self,
        iter: I,
    ) -> AudioBuffers<'a>
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

        // If a realloc occurred, we must rewrite all the pointers.
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

        AudioBuffers {
            buffers: CelledClapAudioBuffer::from_raw_slice(&mut self.buffer_configs[..total]),
            frames_count: if min_channel_buffer_length == usize::MAX {
                None
            } else {
                Some(min_channel_buffer_length as u32)
            },
        }
    }

    #[inline]
    pub fn port_count(&self) -> usize {
        self.buffer_configs.len()
    }
}

#[derive(Copy, Clone)]
pub struct AudioBuffers<'a> {
    buffers: &'a [CelledClapAudioBuffer],
    frames_count: Option<u32>,
}

impl<'a> AudioBuffers<'a> {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            buffers: &[],
            frames_count: None,
        }
    }

    /// Shortens the [`frames_count`] of these input buffers.
    ///
    /// This does not actually change the underlying buffers themselves, it only reduces the
    /// slice that will be exposed to the plugin.
    ///
    /// This method does nothing if `max_buffer_size` is greater or equal than the current [`frames_count`].
    ///
    /// [`frames_count`]: self.frames_count
    pub fn truncate(&mut self, max_buffer_size: u32) {
        if let Some(frames_count) = self.frames_count {
            self.frames_count = Some(frames_count.min(max_buffer_size))
        }
    }

    /// # Safety
    ///
    /// The caller must ensure the given pointers to all buffer structs are valid for 'a,
    /// including all the buffer pointers they themselves contain.
    ///
    /// The caller must also ensure `frames_count` is lower than or equal to the sizes of the
    /// channel buffers pointed to by `buffers`.
    #[inline]
    pub unsafe fn from_raw_buffers(buffers: *mut [clap_audio_buffer], frames_count: u32) -> Self {
        Self {
            buffers: *(buffers as *const _ as *const _),
            frames_count: Some(frames_count),
        }
    }

    #[inline]
    pub fn as_raw_buffers(&self) -> *mut [clap_audio_buffer] {
        CelledClapAudioBuffer::slice_as_raw_ptr(self.buffers)
    }

    /// The number of port buffers this [`AudioBuffers`] has been given.
    #[inline]
    pub fn port_count(&self) -> usize {
        self.buffers.len()
    }

    /// The number of frames in these input buffers.
    ///
    /// This is the minimum frame count of all the port buffers this has been given.
    ///
    /// If this has no port buffer (i.e. [`port_count`](self.port_count) is zero), this returns `None`.
    #[inline]
    pub fn frames_count(&self) -> Option<u32> {
        self.frames_count
    }

    #[inline]
    pub fn port_info(&self, port_index: u32) -> Option<AudioPortProcessingInfo> {
        let info_ptr = self.buffers.get(port_index as usize)?;

        Some(info_ptr.processing_info())
    }

    #[inline]
    pub fn port_infos(&self) -> impl Iterator<Item = AudioPortProcessingInfo> + '_ {
        self.buffers
            .iter()
            .map(CelledClapAudioBuffer::processing_info)
    }

    /// Returns the minimum number of frames available both in this [`AudioBuffers`] and
    /// the given [`AudioBuffers`].
    ///
    /// This is useful to ensure a safe frame count for a `process` batch that would receive those
    /// input and output audio buffers.
    #[inline]
    pub fn min_available_frames_with(&self, outputs: &AudioBuffers) -> u32 {
        match (self.frames_count, outputs.frames_count) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) | (None, Some(a)) => a,
            (None, None) => 0,
        }
    }
}

#[cfg(feature = "clack-plugin")]
impl<'a> AudioBuffers<'a> {
    pub fn from_plugin_audio_mut(
        audio: &'a mut clack_plugin::prelude::Audio,
    ) -> (AudioBuffers<'a>, AudioBuffers<'a>) {
        let frames_count = audio.frames_count();

        // SAFETY: the validity of the buffers is guaranteed by the Audio type
        unsafe {
            (
                AudioBuffers::from_raw_buffers(audio.raw_inputs(), frames_count),
                AudioBuffers::from_raw_buffers(audio.raw_outputs(), frames_count),
            )
        }
    }

    pub fn from_plugin_audio(audio: clack_plugin::prelude::Audio) -> (AudioBuffers, AudioBuffers) {
        let frames_count = audio.frames_count();

        // SAFETY: the validity of the buffers is guaranteed by the Audio type
        unsafe {
            (
                AudioBuffers::from_raw_buffers(audio.raw_inputs(), frames_count),
                AudioBuffers::from_raw_buffers(audio.raw_outputs(), frames_count),
            )
        }
    }

    pub fn as_plugin_audio_with_outputs(&self, outputs: &Self) -> clack_plugin::prelude::Audio<'a> {
        let frames_count = self.min_available_frames_with(outputs);

        // SAFETY: the validity of the buffers is guaranteed by this type
        unsafe {
            clack_plugin::prelude::Audio::from_raw_buffers(
                self.as_raw_buffers(),
                outputs.as_raw_buffers(),
                frames_count,
            )
        }
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
    pub fn audio_buffers_work_with_many_types() {
        let mut ports = AudioPorts::with_capacity(2, 1);
        let mut bufs = [[0f32; 4]; 2];

        let buffers = ports.with_buffers(&mut bufs);

        assert_eq!(buffers.buffers.len(), 1);
        assert_eq!(buffers.frames_count, Some(4));
    }

    #[test]
    pub fn input_audio_buffers_work() {
        let mut ports = AudioPorts::with_capacity(2, 1);
        let mut bufs = [[0f32; 4]; 2];

        let buffers = ports.with_buffers([AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_input_only(bufs.iter_mut().map(|b| InputChannel {
                buffer: b.as_mut_slice(),
                is_constant: false,
            })),
        }]);

        assert_eq!(buffers.buffers.len(), 1);
        assert_eq!(buffers.frames_count, Some(4));
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
        assert_eq!(buffers.frames_count, Some(4));

        assert_eq!(bufs.len(), 2); // Check borrow still works
        assert_eq!(ports.port_count(), 1);
    }

    #[test]
    pub fn input_audio_buffers_work_with_refcell() {
        let mut ports = AudioPorts::with_capacity(2, 1);
        let bufs = [RefCell::new([0f32; 4]), RefCell::new([0f32; 4])];
        let mut borrowed: Vec<_> = bufs.iter().map(|c| c.borrow_mut()).collect();

        let buffers = ports.with_buffers([AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_input_only(borrowed.iter_mut().map(|b| {
                InputChannel {
                    buffer: b.as_mut_slice(),
                    is_constant: false,
                }
            })),
        }]);

        assert_eq!(buffers.buffers.len(), 1);
        assert_eq!(buffers.frames_count, Some(4));
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
        assert_eq!(buffers.frames_count, Some(4));

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
            input_ports.with_buffers(input_bufs.iter_mut().map(|bufs| AudioPortBuffer {
                latency: 0,
                channels: AudioPortBufferType::f32_input_only(bufs.iter_mut().map(|b| {
                    InputChannel {
                        buffer: b.as_mut_slice(),
                        is_constant: false,
                    }
                })),
            }));

        let output_buffers =
            output_ports.with_output_buffers(output_bufs.iter_mut().map(|bufs| AudioPortBuffer {
                latency: 0,
                channels: AudioPortBufferType::f32_output_only(
                    bufs.iter_mut().map(|b| b.as_mut_slice()),
                ),
            }));

        assert_eq!(input_buffers.buffers.len(), 2);
        assert_eq!(input_buffers.frames_count, Some(4));
        assert_eq!(output_buffers.buffers.len(), 2);
        assert_eq!(output_buffers.frames_count, Some(4));

        let process = clap_process {
            audio_inputs: input_buffers.as_raw_buffers().cast(),
            audio_outputs: output_buffers.as_raw_buffers().cast(),
            audio_inputs_count: input_buffers.buffers.len() as u32,
            audio_outputs_count: output_buffers.buffers.len() as u32,

            steady_time: 0,
            frames_count: 4,
            transport: null_mut(),
            in_events: null_mut(),
            out_events: null_mut(),
        };

        // SAFETY: we built the process struct above, it should be good.
        let mut audio = unsafe { Audio::from_raw(&process) };

        for (port, bufs) in audio.input_ports().zip(&input_bufs) {
            let channels = port.channels().unwrap().into_f32().unwrap();

            assert_eq!(channels.channel_count(), 128);
            for (channel, buf) in channels.iter().zip(bufs.iter()) {
                assert_eq!(channel, buf)
            }
        }

        for (mut port, bufs) in audio.output_ports().zip(&output_bufs) {
            let channels = port.channels().unwrap().into_f32().unwrap();

            assert_eq!(channels.channel_count(), 128);
            for (channel, buf) in channels.iter().zip(bufs.iter()) {
                assert_eq!(channel, buf)
            }
        }
    }
}
