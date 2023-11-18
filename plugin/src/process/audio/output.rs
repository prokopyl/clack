use crate::prelude::Audio;
use crate::process::audio::{BufferError, SampleType};
use crate::process::InputChannelsIter;
use clack_common::process::ConstantMask;
use clap_sys::audio_buffer::clap_audio_buffer;
use std::slice::IterMut;

/// An iterator of all of the available [`OutputPort`]s from an [`Audio`] struct.
pub struct OutputPortsIter<'a> {
    outputs: IterMut<'a, clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> OutputPortsIter<'a> {
    #[inline]
    pub(crate) fn new(audio: &'a mut Audio<'_>) -> Self {
        Self {
            outputs: audio.outputs.iter_mut(),
            frames_count: audio.frames_count,
        }
    }
}

impl<'a> Iterator for OutputPortsIter<'a> {
    type Item = OutputPort<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.outputs
            .next()
            .map(|buf| unsafe { OutputPort::from_raw(buf, self.frames_count) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.outputs.size_hint()
    }
}

impl<'a> ExactSizeIterator for OutputPortsIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.outputs.len()
    }
}

/// An output audio port.
pub struct OutputPort<'a> {
    inner: &'a mut clap_audio_buffer,
    frames_count: u32,
}

impl<'a> OutputPort<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(inner: &'a mut clap_audio_buffer, frames_count: u32) -> Self {
        Self {
            inner,
            frames_count,
        }
    }

    /// Retrieves the output port's channels.
    ///
    /// Because each port can hold either [`f32`] or [`f64`] sample data, this method returns a
    /// [`SampleType`] enum of the input channels, to indicate which one the port holds.
    ///
    /// # Errors
    ///
    /// This method returns a [`BufferError::InvalidChannelBuffer`] if the host provided neither
    /// [`f32`] or [`f64`] buffer type, which is invalid per the CLAP specification.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_plugin::process::audio::{OutputChannels, OutputPort, SampleType};
    ///
    /// # fn foo(port: OutputPort) {
    /// let mut port: OutputPort = /* ... */
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
    /// let channels: OutputChannels<f32> = port.channels().unwrap().into_f32().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn channels(
        &mut self,
    ) -> Result<SampleType<OutputChannels<'a, f32>, OutputChannels<'a, f64>>, BufferError> {
        Ok(unsafe { SampleType::from_raw_buffer_mut(self.inner) }?.map(
            |data| OutputChannels {
                data,
                frames_count: self.frames_count,
            },
            |data| OutputChannels {
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

    /// Sets the constant mask for this port.
    #[inline]
    pub fn set_constant_mask(&mut self, new_mask: ConstantMask) {
        self.inner.constant_mask = new_mask.to_bits()
    }
}

/// An [`OutputPort`]'s channels' data buffers, which contains samples of a given type `S`.
///
/// The sample type `S` is always going to be either [`f32`] or [`f64`], as returned by
/// [`OutputPort::channels`].
pub struct OutputChannels<'a, S> {
    pub(crate) frames_count: u32,
    pub(crate) data: &'a mut [*const S],
}

impl<'a, S> OutputChannels<'a, S> {
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
    pub fn raw_data(&self) -> &[*const S] {
        self.data
    }

    /// The number of channels.
    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.data.len() as u32
    }

    /// Retrieves the sample buffer of the channel at a given index, as a read-only slice.
    #[inline]
    pub fn channel(&self, channel_index: u32) -> Option<&[S]> {
        unsafe {
            self.data.get(channel_index as usize).map(|data| {
                core::slice::from_raw_parts(*data as *const _, self.frames_count as usize)
            })
        }
    }

    /// Retrieves the sample buffer of the channel at a given index, as a mutable slice.
    #[inline]
    pub fn channel_mut(&mut self, channel_index: u32) -> Option<&mut [S]> {
        unsafe {
            self.data.get(channel_index as usize).map(|data| {
                core::slice::from_raw_parts_mut(*data as *mut _, self.frames_count as usize)
            })
        }
    }

    /// Gets a read-only iterator over all of the port's channels' sample buffers.
    #[inline]
    pub fn iter(&self) -> InputChannelsIter<S> {
        InputChannelsIter {
            data: self.data.iter(),
            frames_count: self.frames_count,
        }
    }

    /// Gets an iterator over all of the port's channels' writable sample buffers.
    #[inline]
    pub fn iter_mut(&mut self) -> OutputChannelsIter<S> {
        OutputChannelsIter {
            data: self.data.as_mut().iter_mut(),
            frames_count: self.frames_count,
        }
    }

    /// Divides the output channels into two at an index.
    ///
    /// The first will contain all channels with indices from `[0, mid)` (excluding
    /// the index `mid` itself) and the second will contain all channels with
    /// indices from `[mid, channel_count)`.
    ///
    /// Unlike [`slice::split_at_mut`](core::slice::split_at_mut), this method does not panic if
    /// `mid` is larger than `channel_count`.
    /// The second [`OutputChannels`] only be empty in this case.
    #[inline]
    pub fn split_at_mut(&mut self, mid: u32) -> (OutputChannels<S>, OutputChannels<S>) {
        let mid = mid as usize;
        if mid >= self.data.len() {
            return (
                OutputChannels {
                    data: self.data,
                    frames_count: self.frames_count,
                },
                OutputChannels {
                    data: &mut [],
                    frames_count: self.frames_count,
                },
            );
        }
        // PANIC: Checked that mid < len above
        let (left, right) = self.data.split_at_mut(mid);

        (
            OutputChannels {
                data: left,
                frames_count: self.frames_count,
            },
            OutputChannels {
                data: right,
                frames_count: self.frames_count,
            },
        )
    }
}
impl<'a, T> IntoIterator for &'a OutputChannels<'a, T> {
    type Item = &'a [T];
    type IntoIter = InputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut OutputChannels<'a, T> {
    type Item = &'a mut [T];
    type IntoIter = OutputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for OutputChannels<'a, T> {
    type Item = &'a mut [T];
    type IntoIter = OutputChannelsIter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        OutputChannelsIter {
            data: self.data.as_mut().iter_mut(),
            frames_count: self.frames_count,
        }
    }
}

/// An iterator over all of an [`OutputPort`]'s channels' writable sample buffers.
pub struct OutputChannelsIter<'a, T> {
    data: IterMut<'a, *const T>,
    frames_count: u32,
}

impl<'a, T> Iterator for OutputChannelsIter<'a, T> {
    type Item = &'a mut [T];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|ptr| unsafe {
            core::slice::from_raw_parts_mut(*ptr as *mut _, self.frames_count as usize)
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.data.size_hint()
    }
}

impl<'a, S> ExactSizeIterator for OutputChannelsIter<'a, S> {
    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }
}
