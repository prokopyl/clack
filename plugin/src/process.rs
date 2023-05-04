//! Types exposing data and metadata to be used by plugins during audio processing.
//!
//! All of those types are exclusively used in the [`Plugin::process`](crate::plugin::Plugin::process)
//! method. See the [`Plugin`](crate::plugin::Plugin) trait documentation for examples on how these types interact.

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
/// Audio buffers in CLAP follow the following structure:
///
/// * Plugins may have an arbitrary amount of input and output ports;
/// * Each port can hold either 32-bit, or 64-bit floating-point sample data;
/// * Port sample data is split in multiple channels (1 for mono, 2 for stereo, etc.);
/// * Each channel is a raw buffer (i.e. slice) of either [`f32`] or [`f64`] samples.
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

    #[inline]
    pub fn input_port(&self, index: usize) -> Option<InputPort> {
        self.inputs
            .get(index)
            .map(|buf| unsafe { InputPort::from_raw(buf, self.frames_count) })
    }

    #[inline]
    pub fn input_port_count(&self) -> usize {
        self.inputs.len()
    }

    #[inline]
    pub fn input_ports(&self) -> InputPortsIter<'a> {
        InputPortsIter::new(self)
    }

    #[inline]
    pub fn output_port(&mut self, index: usize) -> Option<OutputPort> {
        self.outputs
            .get_mut(index)
            // SAFETY: &mut ensures there is no input being read concurrently
            .map(|buf| unsafe { OutputPort::from_raw(buf, self.frames_count) })
    }

    #[inline]
    pub fn output_port_count(&self) -> usize {
        self.outputs.len()
    }

    #[inline]
    pub fn output_ports(&mut self) -> OutputPortsIter {
        OutputPortsIter::new(self)
    }

    #[inline]
    pub fn port_pairs(&mut self) -> PairedPortsIter {
        PairedPortsIter::new(self)
    }

    #[inline]
    pub fn port_pair(&mut self, index: usize) -> Option<PairedPort> {
        unsafe {
            PairedPort::from_raw(
                self.inputs.get(index),
                self.outputs.get_mut(index),
                self.frames_count,
            )
        }
    }

    #[inline]
    pub fn port_pair_count(&self) -> usize {
        self.input_port_count().max(self.output_port_count())
    }

    #[inline]
    pub fn get_range<R: RangeBounds<usize> + Clone>(&mut self, range: R) -> Audio {
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

    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.frames_count
    }
}

impl<'a> IntoIterator for &'a mut Audio<'a> {
    type Item = PairedPort<'a>;
    type IntoIter = PairedPortsIter<'a>;

    /// Returns a mutable iterator over all port pairs. This is equivalent to using
    /// [`port_pairs`](Audio::port_pairs).
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.port_pairs()
    }
}
