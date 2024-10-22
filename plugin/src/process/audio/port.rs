use crate::prelude::Audio;
use crate::process::audio::{AudioBuffer, BufferError, CelledClapAudioBuffer, SampleType};
use clack_common::process::ConstantMask;
use std::slice::Iter;

/// An iterator of all the available [`Port`]s from an [`Audio`] struct.
pub struct PortsIter<'a> {
    inputs: Iter<'a, CelledClapAudioBuffer>,
    frames_count: u32,
}

impl<'a> PortsIter<'a> {
    #[inline]
    pub(crate) fn new(audio: &Audio<'a>) -> Self {
        Self {
            inputs: audio.inputs.iter(),
            frames_count: audio.frames_count,
        }
    }
}

impl<'a> Iterator for PortsIter<'a> {
    type Item = Port<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inputs
            .next()
            // SAFETY: The Audio type this is built from ensures each buffer is valid
            // and is of length frames_count.
            .map(|buf| unsafe { Port::from_raw(buf, self.frames_count) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inputs.size_hint()
    }
}

impl ExactSizeIterator for PortsIter<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.inputs.len()
    }
}

/// An audio port.
#[derive(Copy, Clone)]
pub struct Port<'a> {
    inner: &'a CelledClapAudioBuffer,
    frames_count: u32,
}

impl<'a> Port<'a> {
    /// # Safety
    ///
    /// * The provided buffer must be valid;
    /// * `frames_count` *must* match the size of the buffers.
    #[inline]
    pub(crate) unsafe fn from_raw(inner: &'a CelledClapAudioBuffer, frames_count: u32) -> Self {
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
    /// use clack_plugin::process::audio::{PortChannels, Port, SampleType};
    ///
    /// # fn foo(port: Port) {
    /// let port: Port = /* ... */
    /// # port;
    ///
    /// // Decide what to do using by matching against every possible configuration
    /// match port.channels().unwrap() {
    ///     SampleType::F32(buf) => { /* Process the 32-bit buffer */ }
    ///     SampleType::F64(buf) => { /* Process the 64-bit buffer */ }
    ///     SampleType::Both(buf32, buf64) => { /* We have both buffers available */ }
    /// }
    ///
    /// // If we're only interested in a single buffer type,
    /// // we can use SampleType's helper methods:
    /// let channels: PortChannels<f32> = port.channels().unwrap().into_f32().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn channels(
        &self,
    ) -> Result<SampleType<PortChannels<'a, f32>, PortChannels<'a, f64>>, BufferError> {
        // SAFETY: this type ensures the provided buffer is valid
        Ok(unsafe { SampleType::from_raw_buffer(self.inner) }?.map(
            |data| PortChannels {
                data,
                frames_count: self.frames_count,
            },
            |data| PortChannels {
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
        ConstantMask::from_bits(self.inner.constant_mask.get())
    }

    /// Sets the constant mask for this port.
    #[inline]
    pub fn set_constant_mask(&self, new_mask: ConstantMask) {
        self.inner.constant_mask.set(new_mask.to_bits())
    }
}

/// An [`Port`]'s channels' data buffers, which contains samples of a given type `S`.
///
/// The sample type `S` is always going to be either [`f32`] or [`f64`], as returned by
/// [`Port::channels`].
#[derive(Copy, Clone)]
pub struct PortChannels<'a, S> {
    frames_count: u32,
    data: &'a [*mut S],
}

impl<'a, S> PortChannels<'a, S> {
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
    pub fn channel(&self, channel_index: u32) -> Option<AudioBuffer<'a, S>> {
        // SAFETY: this type guarantees the buffer pointer is valid and of size frames_count
        unsafe {
            self.data
                .get(channel_index as usize)
                .map(|data| AudioBuffer::from_raw_parts(*data, self.frames_count as usize))
        }
    }

    /// Gets an iterator over all the port's channels' sample buffers.
    #[inline]
    pub fn iter(&self) -> PortChannelsIter<'a, S> {
        PortChannelsIter {
            data: self.data.iter(),
            frames_count: self.frames_count,
        }
    }
}

impl<'a, T> IntoIterator for PortChannels<'a, T> {
    type Item = AudioBuffer<'a, T>;
    type IntoIter = PortChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a PortChannels<'a, T> {
    type Item = AudioBuffer<'a, T>;
    type IntoIter = PortChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over all of an [`Port`]'s channels' sample buffers.
pub struct PortChannelsIter<'a, T> {
    data: Iter<'a, *mut T>,
    frames_count: u32,
}

impl<'a, T> Iterator for PortChannelsIter<'a, T> {
    type Item = AudioBuffer<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.data
            .next()
            // SAFETY: iterator can only get created from a PortChannels, which guarantees
            // the buffer is both valid and of length frames_count
            .map(|ptr| unsafe { AudioBuffer::from_raw_parts(*ptr, self.frames_count as usize) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.data.size_hint()
    }
}

impl<S> ExactSizeIterator for PortChannelsIter<'_, S> {
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}
