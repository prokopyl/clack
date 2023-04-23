use clack_common::events::event_types::TransportEvent;
use clack_common::events::io::{InputEvents, OutputEvents};
use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::process::clap_process;

pub use clack_common::process::ProcessStatus;
pub mod audio;
use audio::*;

#[repr(C)]
pub struct Process {
    inner: clap_process,
}

impl Process {
    #[inline]
    pub(crate) unsafe fn from_raw<'a>(
        raw: *const clap_process,
    ) -> (&'a Process, Audio<'a>, Events<'a>) {
        // SAFETY: Process is repr(C) and is guaranteed to have the same memory representation
        let process: &Process = &*(raw as *const _);
        (process, Audio::from_raw(&*raw), Events::from_raw(&*raw))
    }

    #[inline]
    pub fn frames_count(&self) -> u32 {
        self.inner.frames_count
    }

    #[inline]
    pub fn steady_time(&self) -> i64 {
        self.inner.steady_time
    }

    #[inline]
    pub fn transport(&self) -> &TransportEvent {
        TransportEvent::from_raw_ref(unsafe { &*self.inner.transport })
    }
}

pub struct Events<'a> {
    pub input: &'a InputEvents<'a>,
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
    pub fn port_pairs(&mut self) -> PortsPairIter {
        PortsPairIter::new(self)
    }

    #[inline]
    pub fn port_pair(&mut self, index: usize) -> Option<PortPair> {
        self.mismatched_port_pair(index, index)
    }

    #[inline]
    pub fn port_pair_count(&self) -> usize {
        self.input_port_count().max(self.output_port_count())
    }

    #[inline]
    pub fn mismatched_port_pair(
        &mut self,
        input_index: usize,
        output_index: usize,
    ) -> Option<PortPair> {
        unsafe {
            PortPair::new(
                self.inputs.get(input_index),
                self.outputs.get_mut(output_index),
                self.frames_count,
            )
        }
    }
}

impl<'a> IntoIterator for &'a mut Audio<'a> {
    type Item = PortPair<'a>;
    type IntoIter = PortsPairIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.port_pairs()
    }
}
