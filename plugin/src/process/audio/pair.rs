use crate::process::audio::pair::ChannelPair::*;
use crate::process::audio::{BufferError, InputPort, OutputPort, SampleType};
use crate::process::Audio;
use clack_common::process::ConstantMask;
use clap_sys::audio_buffer::clap_audio_buffer;
use std::slice::{Iter, IterMut};

/// A pair of Input and Output ports.
///
/// Note that in case of asymmetric port layouts (e.g. side-chain), there may not
/// be an associated output port to an input port, or vice-versa. The same goes for
/// paired channels.
///
/// In those cases, a given pair will only contain one port instead of two. However,
/// a [`PortPair`] is always guaranteed to contain at least one port, be it an input or output.
pub struct PortPair<'a> {
    input: Option<&'a clap_audio_buffer>,
    output: Option<&'a mut clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> PortPair<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(
        input: Option<&'a clap_audio_buffer>,
        output: Option<&'a mut clap_audio_buffer>,
        frames_count: u32,
    ) -> Option<Self> {
        match (input, output) {
            (None, None) => None,
            (input, output) => Some(PortPair {
                input,
                output,
                frames_count,
            }),
        }
    }

    /// Gets the [`InputPort`] of this pair.
    ///
    /// If the port layout is asymmetric and there is no input port, this returns [`None`].
    #[inline]
    pub fn input(&self) -> Option<InputPort> {
        self.input
            .map(|i| unsafe { InputPort::from_raw(i, self.frames_count) })
    }

    /// Gets the [`OutputPort`] of this pair.
    ///
    /// If the port layout is asymmetric and there is no output port, this returns [`None`].
    #[inline]
    pub fn output(&mut self) -> Option<OutputPort> {
        self.output
            .as_mut()
            .map(|i| unsafe { OutputPort::from_raw(i, self.frames_count) })
    }

    /// Retrieves the port pair's channels.
    ///
    /// Because each port can hold either [`f32`] or [`f64`] sample data, this method returns a
    /// [`SampleType`] enum of the paired channels, to indicate which one the ports holds.
    ///
    /// # Errors
    ///
    /// This method returns a [`BufferError::InvalidChannelBuffer`] if the host provided neither
    /// [`f32`] or [`f64`] buffer type, which is invalid per the CLAP specification.
    ///
    /// Additionally, if the two port have different buffer sample types (i.e. one holds [`f32`]
    /// and the other holds [`f64`]), then a [`BufferError::MismatchedBufferPair`] error is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_plugin::process::audio::{PairedChannels, PortPair, SampleType};
    ///
    /// # fn foo(port: PortPair) {
    /// let mut port: PortPair = /* ... */
    /// # port;
    ///
    /// // Decide what to do using by matching against every possible configuration
    /// match port.channels().unwrap() {
    ///     SampleType::F32(buf) => { /* Process the 32-bit buffers */ },
    ///     SampleType::F64(buf) => { /* Process the 64-bit buffers */ },
    ///     SampleType::Both(buf32, buf64) => { /* We have both types of buffers available */ }
    /// }
    ///
    /// // If we're only interested in a single buffer type,
    /// // we can use SampleType's helper methods:
    /// let channels: PairedChannels<f32> = port.channels().unwrap().into_f32().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn channels(
        &mut self,
    ) -> Result<SampleType<PairedChannels<'a, f32>, PairedChannels<'a, f64>>, BufferError> {
        let input = match self.input {
            None => SampleType::Both([].as_slice(), [].as_slice()),
            Some(buffer) => unsafe { SampleType::from_raw_buffer(buffer)? },
        };

        let output = match self.output.as_mut() {
            None => SampleType::Both([].as_mut_slice(), [].as_mut_slice()),
            Some(buffer) => unsafe { SampleType::from_raw_buffer_mut(buffer)? },
        };

        Ok(input.try_match_with(output)?.map(
            |(i, o)| PairedChannels {
                input_data: i,
                output_data: o,
                frames_count: self.frames_count,
            },
            |(i, o)| PairedChannels {
                input_data: i,
                output_data: o,
                frames_count: self.frames_count,
            },
        ))
    }

    /// The number of channels in this port pair.
    ///
    /// Since there may be more channels in one port than in the other, this method also counts
    /// the partial [`ChannelPair`]s that can be returned, and therefore returns the maximum number
    /// of channels between the two ports.  
    #[inline]
    pub fn channel_pair_count(&self) -> usize {
        let in_channels = self.input.map(|b| b.channel_count).unwrap_or(0);
        let out_channels = self.output.as_ref().map(|b| b.channel_count).unwrap_or(0);

        in_channels.max(out_channels) as usize
    }

    /// Returns the number of frames to process in this block.
    ///
    /// This will always match the number of samples of every audio channel buffer. The two ports
    /// *cannot* have different buffer sizes.
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    /// The latency from and to the audio interface for this port pair, in samples.
    ///
    /// This returns a tuple containing the latenciess for the input and output port,
    /// respectively.
    ///
    /// If one port isn't present in this pair, then [`None`] is returned.
    #[inline]
    pub fn latencies(&self) -> (Option<u32>, Option<u32>) {
        (
            self.input.map(|i| i.latency),
            self.output.as_ref().map(|o| o.latency),
        )
    }

    /// The [`ConstantMask`]s for the two ports.
    ///
    /// This returns a tuple containing the [`ConstantMask`]s for the input and output port,
    /// respectively.
    ///
    /// If one port isn't present in this pair, then [`ConstantMask::FULLY_CONSTANT`] is returned.
    #[inline]
    pub fn constant_masks(&self) -> (ConstantMask, ConstantMask) {
        (
            self.input
                .map(|i| ConstantMask::from_bits(i.constant_mask))
                .unwrap_or(ConstantMask::FULLY_CONSTANT),
            self.output
                .as_ref()
                .map(|o| ConstantMask::from_bits(o.constant_mask))
                .unwrap_or(ConstantMask::FULLY_CONSTANT),
        )
    }
}

/// An [`PortPair`]'s channels' data buffers, which contains samples of a given type `S`.
///
/// The sample type `S` is always going to be either [`f32`] or [`f64`], as returned by
/// [`PortPair::channels`].
pub struct PairedChannels<'a, S> {
    input_data: &'a [*mut S],
    output_data: &'a mut [*mut S],
    frames_count: u32,
}

impl<'a, S> PairedChannels<'a, S> {
    /// Returns the number of frames to process in this block.
    ///
    /// This will always match the number of samples of every audio channel buffer, from both
    /// the input and output port.
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }

    /// The number of input channels.
    #[inline]
    pub fn input_channel_count(&self) -> usize {
        self.input_data.len()
    }

    /// The number of output channels.
    #[inline]
    pub fn output_channel_count(&self) -> usize {
        self.output_data.len()
    }

    /// The total number of channel pairs.
    ///
    /// Since there may be more channels in one port than in the other, this method also counts
    /// the partial [`ChannelPair`]s that can be returned, and therefore returns the maximum number
    /// of channels between the input and output ports.  
    #[inline]
    pub fn channel_pair_count(&self) -> usize {
        self.input_channel_count().max(self.output_channel_count())
    }

    /// Retrieves the pair of sample buffers for the pair of channels at a given index.
    ///
    /// If there is no channel at the given index (i.e. `channel_index` is greater or equal than
    /// [`channel_pair_count`](Self::channel_pair_count)), this returns [`None`].
    ///
    /// See [`ChannelPair`]'s documentation for examples on how to access sample buffers from it.
    #[inline]
    pub fn channel_pair(&mut self, index: usize) -> Option<ChannelPair<'a, S>> {
        let input = self
            .input_data
            .get(index)
            .map(|ptr| unsafe { core::slice::from_raw_parts(*ptr, self.frames_count as usize) });

        let output = self.output_data.get(index).map(|ptr| unsafe {
            core::slice::from_raw_parts_mut(*ptr, self.frames_count as usize)
        });

        ChannelPair::from_optional_io(input, output)
    }

    /// Gets an iterator over all of the ports' [`ChannelPair`]s.
    #[inline]
    pub fn iter_mut(&mut self) -> PairedChannelsIter<S> {
        PairedChannelsIter {
            input_iter: self.input_data.iter(),
            output_iter: self.output_data.iter_mut(),
            frames_count: self.frames_count,
        }
    }
}

impl<'a, S> IntoIterator for PairedChannels<'a, S> {
    type Item = ChannelPair<'a, S>;
    type IntoIter = PairedChannelsIter<'a, S>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        PairedChannelsIter {
            input_iter: self.input_data.iter(),
            output_iter: self.output_data.iter_mut(),
            frames_count: self.frames_count,
        }
    }
}

/// An iterator over all of a [`PortPair`]'s [`ChannelPair`]s.
pub struct PairedChannelsIter<'a, S> {
    input_iter: Iter<'a, *mut S>,
    output_iter: IterMut<'a, *mut S>,
    frames_count: u32,
}

impl<'a, S> Iterator for PairedChannelsIter<'a, S> {
    type Item = ChannelPair<'a, S>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let input = self
            .input_iter
            .next()
            .map(|ptr| unsafe { core::slice::from_raw_parts(*ptr, self.frames_count as usize) });

        let output = self.output_iter.next().map(|ptr| unsafe {
            core::slice::from_raw_parts_mut((*ptr) as *mut _, self.frames_count as usize)
        });

        ChannelPair::from_optional_io(input, output)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, S> ExactSizeIterator for PairedChannelsIter<'a, S> {
    #[inline]
    fn len(&self) -> usize {
        self.input_iter.len().max(self.output_iter.len())
    }
}

/// An iterator of all of the available [`PortPair`]s from an [`Audio`] struct.
pub struct PortPairsIter<'a> {
    inputs: Iter<'a, clap_audio_buffer>,
    outputs: IterMut<'a, clap_audio_buffer>,
    frames_count: u32,
}

impl<'a> PortPairsIter<'a> {
    #[inline]
    pub(crate) fn new(audio: &'a mut Audio<'_>) -> Self {
        Self {
            inputs: audio.inputs.iter(),
            outputs: audio.outputs.iter_mut(),
            frames_count: audio.frames_count,
        }
    }
}

impl<'a> Iterator for PortPairsIter<'a> {
    type Item = PortPair<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { PortPair::from_raw(self.inputs.next(), self.outputs.next(), self.frames_count) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> ExactSizeIterator for PortPairsIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inputs.len().max(self.outputs.len())
    }
}

/// A pair of input and output channel buffers.
///
/// Because port and channel layouts may differ between inputs and outputs, a [`ChannelPair`]
/// may actually contain only an input or an output instead of both.
///
/// This enum also allows to check for the fairly common case where Host may re-use the same
/// buffer for both the input and output, and expect the plugin to do in-place processing.
pub enum ChannelPair<'a, S> {
    /// There is only an input channel present, there was no matching output.
    InputOnly(&'a [S]),
    /// There is only an output channel present, there was no matching input.
    OutputOnly(&'a mut [S]),
    /// Both the input and output channels are present, and available separately.
    InputOutput(&'a [S], &'a mut [S]),
    /// Both the input and output channels are present, but they actually share the same buffer.
    ///
    /// In this case, the slice is already filled with the input channel's data, and the host
    /// considers the contents of this buffer after processing to be the output channel's data.
    InPlace(&'a mut [S]),
}

impl<'a, S> ChannelPair<'a, S> {
    #[inline]
    pub(crate) fn from_optional_io(
        input: Option<&'a [S]>,
        output: Option<&'a mut [S]>,
    ) -> Option<ChannelPair<'a, S>> {
        match (input, output) {
            (None, None) => None,
            (Some(input), None) => Some(InputOnly(input)),
            (None, Some(output)) => Some(OutputOnly(output)),
            (Some(input), Some(output)) => Some(if input.as_ptr() == output.as_ptr() {
                InPlace(output)
            } else {
                InputOutput(input, output)
            }),
        }
    }

    /// Attempts to retrieve the input channel's buffer data, if the input channel is present.
    #[inline]
    pub fn input(&'a self) -> Option<&'a [S]> {
        match self {
            InputOnly(i) | InputOutput(i, _) => Some(i),
            OutputOnly(_) => None,
            InPlace(io) => Some(io),
        }
    }

    /// Attempts to retrieve a read-only reference to the output channel's buffer data,
    /// if the output channel is present.
    #[inline]
    pub fn output(&'a self) -> Option<&'a [S]> {
        match self {
            OutputOnly(o) | InputOutput(_, o) | InPlace(o) => Some(o),
            InputOnly(_) => None,
        }
    }

    /// Attempts to retrieve a read-write reference to the output channel's buffer data,
    /// if the output channel is present.
    #[inline]
    pub fn output_mut(&'a mut self) -> Option<&'a mut [S]> {
        match self {
            OutputOnly(o) | InputOutput(_, o) | InPlace(o) => Some(o),
            InputOnly(_) => None,
        }
    }
}
