use crate::internal_utils::slice_from_external_parts;
use crate::prelude::Audio;
use crate::process::audio::{BufferError, SampleType};
use clack_common::process::ConstantMask;
use clap_sys::audio_buffer::clap_audio_buffer;
use std::slice::Iter;

/// An iterator of all the available [`InputPort`]s from an [`Audio`] struct.
pub struct InputPortsIter<'a> {
    inputs: Iter<'a, clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> InputPortsIter<'a> {
    #[inline]
    pub(crate) fn new(audio: &Audio<'a>) -> Self {
        Self {
            inputs: audio.inputs.iter(),
            frames_count: audio.frames_count,
        }
    }
}

impl<'a> Iterator for InputPortsIter<'a> {
    type Item = InputPort<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inputs
            .next()
            // SAFETY: The Audio type this is built from ensures each buffer is valid
            // and is of length frames_count.
            .map(|buf| unsafe { InputPort::from_raw(buf, self.frames_count) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inputs.size_hint()
    }
}

impl<'a> ExactSizeIterator for InputPortsIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inputs.len()
    }
}

/// An input audio port.
#[derive(Copy, Clone)]
pub struct InputPort<'a> {
    inner: &'a clap_audio_buffer,
    frames_count: u32,
}

impl<'a> InputPort<'a> {
    /// # Safety
    ///
    /// * The provided buffer must be valid;
    /// * `frames_count` *must* match the size of the buffers.
    #[inline]
    pub(crate) unsafe fn from_raw(inner: &'a clap_audio_buffer, frames_count: u32) -> Self {
        Self {
            inner,
            frames_count,
        }
    }

    /// Retrieves the input port's channels.
    ///
    /// Because each port can hold either [`f32`] or [`f64`] sample data, this method returns a
    /// [`SampleType`] enum of the input channels, to indicate which one the port holds.
    ///
    /// # Errors
    ///
    /// This method returns a [`BufferError::InvalidChannelBuffer`] if the host provided neither
    /// [`f32`] nor [`f64`] buffer type, which is invalid per the CLAP specification.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_plugin::process::audio::{InputChannels, InputPort, SampleType};
    ///
    /// # fn foo(port: InputPort) {
    /// let port: InputPort = /* ... */
    /// # port;
    ///
    /// // Decide what to do using by matching against every possible configuration
    /// match port.channels().unwrap() {
    ///     SampleType::F32(buf) => { /* Process the 32-bit buffer */ },
    ///     SampleType::F64(buf) => { /* Process the 64-bit buffer */ },
    ///     SampleType::Both(buf32, buf64) => { /* We have both buffers available */ }
    /// }
    ///
    /// // If we're only interested in a single buffer type,
    /// // we can use SampleType's helper methods:
    /// let channels: InputChannels<f32> = port.channels().unwrap().into_f32().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn channels(
        &self,
    ) -> Result<SampleType<InputChannels<'a, f32>, InputChannels<'a, f64>>, BufferError> {
        // SAFETY: this type ensures the provided buffer is valid
        Ok(unsafe { SampleType::from_raw_buffer(self.inner) }?.map(
            |data| InputChannels {
                data,
                frames_count: self.frames_count,
            },
            |data| InputChannels {
                data,
                frames_count: self.frames_count,
            },
        ))
    }

    /// Returns the number of frames to process in this block.
    ///
    /// This will always match the number of samples of every audio channel buffer.
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    /// The number of channels in this port.
    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.inner.channel_count
    }

    /// The latency from the audio interface for this port, in samples.
    #[inline]
    pub fn latency(&self) -> u32 {
        self.inner.latency
    }

    /// The constant mask for this port.
    #[inline]
    pub fn constant_mask(&self) -> ConstantMask {
        ConstantMask::from_bits(self.inner.constant_mask)
    }
}

/// An [`InputPort`]'s channels' data buffers, which contains samples of a given type `S`.
///
/// The sample type `S` is always going to be either [`f32`] or [`f64`], as returned by
/// [`InputPort::channels`].
#[derive(Copy, Clone)]
pub struct InputChannels<'a, S> {
    frames_count: u32,
    data: &'a [*mut S],
}

impl<'a, S> InputChannels<'a, S> {
    /// Returns the number of frames to process in this block.
    ///
    /// This will always match the number of samples of every audio channel buffer.
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    /// Returns the raw pointer data, as provided by the host.
    ///
    /// In CLAP's API, hosts provide a port's audio data as an array of raw pointers, each of which points
    /// to the start of a sample array of type `S` and of [`frames_count`](Self::frames_count) length.
    #[inline]
    pub fn raw_data(&self) -> &'a [*mut S] {
        self.data
    }

    /// The number of channels.
    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.data.len() as u32
    }

    /// Retrieves the sample buffer of the channel at a given index.
    ///
    /// If there is no channel at the given index (i.e. `channel_index` is greater or equal than
    /// [`channel_count`](Self::channel_count)), this returns [`None`].
    #[inline]
    pub fn channel(&self, channel_index: u32) -> Option<&'a [S]> {
        // SAFETY: this type guarantees the buffer pointer is valid and of size frames_count
        unsafe {
            self.data
                .get(channel_index as usize)
                .map(|data| slice_from_external_parts(*data, self.frames_count as usize))
        }
    }

    /// Gets an iterator over all the port's channels' sample buffers.
    #[inline]
    pub fn iter(&self) -> InputChannelsIter<'a, S> {
        InputChannelsIter {
            data: self.data.iter(),
            frames_count: self.frames_count,
        }
    }
}

impl<'a, T> IntoIterator for InputChannels<'a, T> {
    type Item = &'a [T];
    type IntoIter = InputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a InputChannels<'a, T> {
    type Item = &'a [T];
    type IntoIter = InputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over all of an [`InputPort`]'s channels' sample buffers.
pub struct InputChannelsIter<'a, T> {
    pub(crate) data: Iter<'a, *mut T>,
    pub(crate) frames_count: u32,
}

impl<'a, T> Iterator for InputChannelsIter<'a, T> {
    type Item = &'a [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.data
            .next()
            // SAFETY: iterator can only get created from an InputChannels, which guarantees
            // the buffer is both valid and of length frames_count
            .map(|ptr| unsafe { slice_from_external_parts(*ptr, self.frames_count as usize) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.data.size_hint()
    }
}

impl<'a, S> ExactSizeIterator for InputChannelsIter<'a, S> {
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}
