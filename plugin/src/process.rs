//! Types exposing data and metadata to be used by plugins during audio processing.
//!
//! All of those types are exclusively used in the [`Plugin::process`](crate::plugin::Plugin::process)
//! method. See the [`Plugin`](crate::plugin::Plugin) trait documentation for examples on how these types interact.

#![deny(missing_docs)]

use clack_common::events::event_types::TransportEvent;
use clack_common::events::io::{InputEvents, OutputEvents};
use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::process::clap_process;
use std::ops::RangeBounds;

pub use clack_common::process::ProcessStatus;
pub mod audio;
use audio::*;

/// Metadata about the current process call.
///
/// This exposes [transport information](Process::transport) (in the form of a [`TransportEvent`]), as well as a
/// [steady sample time counter](Process::steady_time).
///
#[derive(Copy, Clone)]
pub struct Process<'a> {
    /// Transport information at sample 0.
    ///
    /// If this is set to [`None`], then this means the plugin is running is a free-running host,
    /// and no transport events will be provided.
    pub transport: Option<&'a TransportEvent>,
    /// A steady sample time counter.
    ///
    /// This field can be used to calculate the sleep duration between two process calls.
    /// This value may be specific to this plugin instance and have no relation to what
    /// other plugin instances may receive.
    ///
    /// If no steady sample time counter is available from the host, this is set to [`None`].
    ///
    /// Note that this counter's maximum value is actually [`i64::MAX`], due to how it is
    /// implemented in the CLAP specification.
    pub steady_time: Option<u64>,
}

impl<'a> Process<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(raw: *const clap_process) -> Process<'a> {
        let transport = (*raw).transport;
        let steady_time = (*raw).steady_time;

        Self {
            steady_time: if steady_time < 0 {
                None
            } else {
                Some(steady_time as u64)
            },
            transport: if transport.is_null() {
                None
            } else {
                Some(TransportEvent::from_raw_ref(&*transport))
            },
        }
    }
}

/// Input and output events that occurred during this processing block.
pub struct Events<'a> {
    /// The input event buffer, for the plugin to read events from.
    pub input: &'a InputEvents<'a>,
    /// The output event buffer, for the plugin to push events into.
    pub output: &'a mut OutputEvents<'a>,
}

impl<'a> Events<'a> {
    pub(crate) unsafe fn from_raw(process: &clap_process) -> Self {
        Self {
            input: InputEvents::from_raw(&*process.in_events),
            output: OutputEvents::from_raw_mut(&mut *(process.out_events as *mut _)),
        }
    }
}

/// Input and output audio buffers to processed by the plugin.
///
/// Audio data in CLAP follow the following tree structure:
///
/// * Plugins may have an arbitrary amount of input and output ports;
/// * Each port can hold either 32-bit, or 64-bit floating-point sample data;
/// * Port sample data is split in multiple channels (1 for mono, 2 for stereo, etc.);
/// * Each channel is a raw buffer (i.e. slice) of either [`f32`] or [`f64`] samples.
///
/// This structure applies both to inputs and outputs: the [`Audio`] struct allows to retrieve
/// [`InputPort`]s and [`OutputPort`]s separately, but they can also be accessed together as
/// Input/Output [`PortPair`]s. This allows for the common use-case of borrowing both an input
/// and its matching output for processing, while also being safe to hosts using the same buffer for
/// both.
///
/// For each port type (input, output, or paired), ports can be accessed either individually with
/// an index, or all at once with an iterator. For instance, [`InputPort`]s can be accessed either
/// one-at-a-time with [`Audio::input_port`], or with an iterator from [`Audio::input_ports`]. A
/// [`Audio::input_port_count`] method is also available. The same methods are available for
/// [`OutputPort`]s and [`PortPair`]s.
///
/// Note that because ports can individually hold either 32-bit or 64-bit sample data, an extra
/// sample type detection step is necessary before the port's channels themselves can be accessed.
/// This is done through the [`channels`](InputPort::channels) methods on each port type, and
/// returns a [`SampleType`] enum indicating whether the port's buffers hold 32-bit or 64-bit samples.
///
/// # Example
///
/// The following example implements a gain plugin that amplifies every input channel by `2`, and
/// writes the result to a matching output channel.
///
/// ```
/// use clack_plugin::prelude::*;
///
/// pub fn process(mut audio: Audio) -> Result<ProcessStatus, PluginError> {
///     for mut port_pair in &mut audio {
///         // For this example, we'll only care about 32-bit sample data.
///         let Some(channel_pairs) = port_pair.channels()?.into_f32() else { continue };
///
///         for channel_pair in channel_pairs {
///             match channel_pair {
///                 // If this input has no matching output, we simply do nothing with it.
///                 ChannelPair::InputOnly(_) => {}
///                 // If this output has no matching input, we fill it with silence.
///                 ChannelPair::OutputOnly(buf) => buf.fill(0.0),
///                 // If this is a separate pair of I/O buffers,
///                 // we can copy the input to the outputs while multiplying
///                 ChannelPair::InputOutput(input, output) => {
///                     for (input, output) in input.iter().zip(output) {
///                         *output = input * 2.0
///                     }
///                 }
///                 // If the host sent us a single buffer to be processed in-place
///                 // (i.e. input and output buffers point to the same location),
///                 // then we can do the processing in-place directly.
///                 ChannelPair::InPlace(buf) => {
///                     for sample in buf {
///                         *sample *= 2.0
///                     }
///                 }
///             }
///         }
///     }
///
///     Ok(ProcessStatus::Continue)
/// }
/// ```
///
pub struct Audio<'a> {
    inputs: &'a [clap_audio_buffer],
    outputs: &'a mut [clap_audio_buffer],
    frames_count: u32,
}

impl<'a> Audio<'a> {
    #[inline]
    pub(crate) unsafe fn from_raw(process: &clap_process) -> Audio {
        unsafe {
            Audio {
                frames_count: process.frames_count,
                inputs: core::slice::from_raw_parts(
                    process.audio_inputs,
                    process.audio_inputs_count as usize,
                ),
                outputs: core::slice::from_raw_parts_mut(
                    process.audio_outputs,
                    process.audio_outputs_count as usize,
                ),
            }
        }
    }

    /// Retrieves the [`InputPort`] at a given index.
    ///
    /// This returns [`None`] if there is no input port at the given index.
    ///
    /// See also the [`input_port_count`](Audio::input_port_count) method to know how many input
    /// ports are available, and the [`input_ports`](Audio::input_ports) method to get all input ports at once.
    #[inline]
    pub fn input_port(&self, index: usize) -> Option<InputPort> {
        self.inputs
            .get(index)
            .map(|buf| unsafe { InputPort::from_raw(buf, self.frames_count) })
    }

    /// Retrieves the number of available [`InputPort`]s.
    #[inline]
    pub fn input_port_count(&self) -> usize {
        self.inputs.len()
    }

    /// Returns an iterator of all the available [`InputPort`]s at once.
    ///
    /// See also the [`input_port`](Audio::input_port) method to retrieve a single input port by
    /// its index.
    #[inline]
    pub fn input_ports(&self) -> InputPortsIter {
        InputPortsIter::new(self)
    }

    /// Retrieves the [`OutputPort`] at a given index.
    ///
    /// This returns [`None`] if there is no input port at the given index.
    ///
    /// See also the [`output_port_count`](Audio::output_port_count) method to know how many input
    /// ports are available, and the [`output_ports`](Audio::output_ports) method to get all input ports at once.
    #[inline]
    pub fn output_port(&mut self, index: usize) -> Option<OutputPort> {
        self.outputs
            .get_mut(index)
            // SAFETY: &mut ensures there is no input being read concurrently
            .map(|buf| unsafe { OutputPort::from_raw(buf, self.frames_count) })
    }

    /// Retrieves the number of available [`OutputPort`]s.
    #[inline]
    pub fn output_port_count(&self) -> usize {
        self.outputs.len()
    }

    /// Returns an iterator of all the available [`OutputPort`]s at once.
    ///
    /// See also the [`output_port`](Audio::output_port) method to retrieve a single input port by
    /// its index.
    #[inline]
    pub fn output_ports(&mut self) -> OutputPortsIter {
        OutputPortsIter::new(self)
    }

    /// Retrieves the [`PortPair`] at a given index.
    ///
    /// This returns [`None`] if there is no available port at the given index.
    ///
    /// See also the [`port_pair_count`](Audio::port_pair_count) method to know how many port
    /// port pairs are available, and the [`port_pairs`](Audio::port_pairs) method to get all port pairs at once.
    #[inline]
    pub fn port_pair(&mut self, index: usize) -> Option<PortPair> {
        unsafe {
            PortPair::from_raw(
                self.inputs.get(index),
                self.outputs.get_mut(index),
                self.frames_count,
            )
        }
    }

    /// Retrieves the number of available [`PortPair`]s.
    ///
    /// Because [`PortPair`] can be mismatched (i.e. have an input but no output, or vice-versa),
    /// this is effectively equal to the maximum number of ports available, either on the input side
    /// or the output side.
    #[inline]
    pub fn port_pair_count(&self) -> usize {
        self.input_port_count().max(self.output_port_count())
    }

    /// Returns an iterator of all the available [`PortPair`]s at once.
    ///
    /// See also the [`port_pair`](Audio::port_pair) method to retrieve a single input port by
    /// its index.
    #[inline]
    pub fn port_pairs(&mut self) -> PortPairsIter {
        PortPairsIter::new(self)
    }

    /// Returns a sub-range of ports as a new [`Audio`] struct, similar to a subslice of items.
    #[inline]
    pub fn port_sub_range<R: RangeBounds<usize> + Clone>(&mut self, range: R) -> Audio {
        let inputs = self
            .inputs
            .get((range.start_bound().cloned(), range.end_bound().cloned()))
            .unwrap_or(&[]);

        let outputs = self
            .outputs
            .get_mut((range.start_bound().cloned(), range.end_bound().cloned()))
            .unwrap_or(&mut []);

        Audio {
            inputs,
            outputs,
            frames_count: self.frames_count,
        }
    }

    /// Returns the number of frames to process in this block.
    ///
    /// This will always match the number of samples of every audio buffer in this [`Audio`] struct.
    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }
}

impl<'buf: 'a, 'a> IntoIterator for &'a mut Audio<'buf> {
    type Item = PortPair<'a>;
    type IntoIter = PortPairsIter<'a>;

    /// Returns an iterator of all the available [`PortPair`]s at once. This is equivalent to using
    /// [`port_pairs`](Audio::port_pairs).
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.port_pairs()
    }
}
