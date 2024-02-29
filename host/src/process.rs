use self::audio_buffers::InputAudioBuffers;
use crate::host::Host;
use crate::host::HostError;
use crate::plugin::{PluginAudioProcessorHandle, PluginSharedHandle};
use crate::prelude::OutputAudioBuffers;
use crate::process::PluginAudioProcessor::*;
use clack_common::events::event_types::TransportEvent;
use clack_common::events::io::{InputEvents, OutputEvents};
use clap_sys::process::clap_process;
use std::cell::UnsafeCell;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::plugin::instance::PluginInstanceInner;
pub use clack_common::process::*;

pub mod audio_buffers;

pub struct PluginAudioConfiguration {
    pub sample_rate: f64,
    pub frames_count_range: RangeInclusive<u32>,
}

pub enum PluginAudioProcessor<H: Host> {
    Started(StartedPluginAudioProcessor<H>),
    Stopped(StoppedPluginAudioProcessor<H>),
    Poisoned,
}

impl<'a, H: 'a + Host> PluginAudioProcessor<H> {
    #[inline]
    pub fn as_started(&self) -> Result<&StartedPluginAudioProcessor<H>, HostError> {
        match self {
            Started(s) => Ok(s),
            Stopped(_) => Err(HostError::ProcessingStopped),
            Poisoned => Err(HostError::ProcessorHandlePoisoned),
        }
    }

    #[inline]
    pub fn as_started_mut(&mut self) -> Result<&mut StartedPluginAudioProcessor<H>, HostError> {
        match self {
            Started(s) => Ok(s),
            Stopped(_) => Err(HostError::ProcessingStopped),
            Poisoned => Err(HostError::ProcessorHandlePoisoned),
        }
    }

    #[inline]
    pub fn as_stopped(&self) -> Result<&StoppedPluginAudioProcessor<H>, HostError> {
        match self {
            Stopped(s) => Ok(s),
            Started(_) => Err(HostError::ProcessingStarted),
            Poisoned => Err(HostError::ProcessorHandlePoisoned),
        }
    }

    #[inline]
    pub fn as_stopped_mut(&mut self) -> Result<&mut StoppedPluginAudioProcessor<H>, HostError> {
        match self {
            Stopped(s) => Ok(s),
            Started(_) => Err(HostError::ProcessingStarted),
            Poisoned => Err(HostError::ProcessorHandlePoisoned),
        }
    }

    #[inline]
    pub fn shared_host_data(&self) -> &<H as Host>::Shared<'_> {
        match self {
            Poisoned => panic!("Plugin audio processor was poisoned"),
            Started(s) => s.shared_host_data(),
            Stopped(s) => s.shared_host_data(),
        }
    }

    #[inline]
    pub fn audio_processor_host_data(&self) -> &<H as Host>::AudioProcessor<'_> {
        match self {
            Poisoned => panic!("Plugin audio processor was poisoned"),
            Started(s) => s.audio_processor_host_data(),
            Stopped(s) => s.audio_processor_host_data(),
        }
    }

    #[inline]
    pub fn audio_processor_host_data_mut(&mut self) -> &mut <H as Host>::AudioProcessor<'_> {
        match self {
            Poisoned => panic!("Plugin audio processor was poisoned"),
            Started(s) => s.audio_processor_host_data_mut(),
            Stopped(s) => s.audio_processor_host_data_mut(),
        }
    }

    pub fn is_started(&self) -> bool {
        match self {
            Poisoned => false,
            Started(_) => true,
            Stopped(_) => false,
        }
    }

    pub fn ensure_processing_started(
        &mut self,
    ) -> Result<&mut StartedPluginAudioProcessor<H>, HostError> {
        match self {
            Started(s) => Ok(s),
            _ => self.start_processing(),
        }
    }

    pub fn start_processing(&mut self) -> Result<&mut StartedPluginAudioProcessor<H>, HostError> {
        let inner = core::mem::replace(self, Poisoned);

        match inner {
            Poisoned => Err(HostError::ProcessorHandlePoisoned),
            Started(s) => {
                *self = Started(s);
                Err(HostError::ProcessingStarted)
            }
            Stopped(s) => match s.start_processing() {
                Ok(s) => {
                    *self = Started(s);
                    Ok(match self {
                        Started(s) => s,
                        _ => unreachable!(),
                    })
                }
                Err(e) => {
                    *self = Stopped(e.processor);
                    Err(HostError::StartProcessingFailed)
                }
            },
        }
    }

    pub fn ensure_processing_stopped(
        &mut self,
    ) -> Result<&mut StoppedPluginAudioProcessor<H>, HostError> {
        match self {
            Stopped(s) => Ok(s),
            _ => self.stop_processing(),
        }
    }

    pub fn stop_processing(&mut self) -> Result<&mut StoppedPluginAudioProcessor<H>, HostError> {
        let inner = core::mem::replace(self, Poisoned);

        match inner {
            Poisoned => Err(HostError::ProcessorHandlePoisoned),
            Stopped(s) => {
                *self = Stopped(s);
                Err(HostError::ProcessingStopped)
            }
            Started(s) => {
                *self = Stopped(s.stop_processing());
                Ok(match self {
                    Stopped(s) => s,
                    _ => unreachable!(),
                })
            }
        }
    }
}

impl<'a, H: 'a + Host> From<StartedPluginAudioProcessor<H>> for PluginAudioProcessor<H> {
    #[inline]
    fn from(p: StartedPluginAudioProcessor<H>) -> Self {
        Started(p)
    }
}

impl<'a, H: 'a + Host> From<StoppedPluginAudioProcessor<H>> for PluginAudioProcessor<H> {
    #[inline]
    fn from(p: StoppedPluginAudioProcessor<H>) -> Self {
        Stopped(p)
    }
}

pub struct StartedPluginAudioProcessor<H: Host> {
    inner: Option<Arc<PluginInstanceInner<H>>>,
    _no_sync: PhantomData<UnsafeCell<()>>,
}

impl<H: Host> StartedPluginAudioProcessor<H> {
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        &mut self,
        audio_inputs: &InputAudioBuffers,
        audio_outputs: &mut OutputAudioBuffers,
        events_input: &InputEvents,
        events_output: &mut OutputEvents,
        steady_time: i64,
        max_frame_count: Option<usize>,
        transport: Option<&TransportEvent>,
    ) -> Result<ProcessStatus, HostError> {
        let min_input_sample_count = audio_inputs.min_channel_buffer_length();
        let min_output_sample_count = audio_outputs.min_channel_buffer_length();

        let mut frames_count = min_input_sample_count.min(min_output_sample_count);
        if let Some(max_frame_count) = max_frame_count {
            frames_count = frames_count.min(max_frame_count)
        }

        let process = clap_process {
            steady_time,
            frames_count: frames_count as u32,
            transport: transport
                .map(|e| e.as_raw_ref() as *const _)
                .unwrap_or(core::ptr::null()),
            audio_inputs_count: audio_inputs.as_raw_buffers().len() as u32,
            audio_outputs_count: audio_outputs.as_raw_buffers().len() as u32,
            audio_inputs: audio_inputs.as_raw_buffers().as_ptr(),
            audio_outputs: audio_outputs.as_raw_buffers().as_mut_ptr(),
            in_events: events_input.as_raw(),
            out_events: events_output.as_raw_mut() as *mut _,
        };

        let instance = self.inner.as_ref().unwrap().raw_instance();

        // SAFETY: this type ensures the function pointer is valid
        let status = ProcessStatus::from_raw(unsafe {
            instance.process.ok_or(HostError::NullProcessFunction)?(instance, &process)
        })
        .ok_or(())
        .and_then(|r| r)
        .map_err(|_| HostError::ProcessingFailed)?;

        Ok(status)
    }

    #[inline]
    pub fn stop_processing(mut self) -> StoppedPluginAudioProcessor<H> {
        let inner = self.inner.take().unwrap();
        // SAFETY: this is called on the audio thread
        unsafe { inner.stop_processing() };

        StoppedPluginAudioProcessor {
            inner,
            _no_sync: PhantomData,
        }
    }

    #[inline]
    pub fn shared_host_data(&self) -> &<H as Host>::Shared<'_> {
        self.inner.as_ref().unwrap().wrapper().shared()
    }

    #[inline]
    pub fn audio_processor_host_data(&self) -> &<H as Host>::AudioProcessor<'_> {
        // SAFETY: we take &self, the only reference to the wrapper on the audio thread, therefore
        // we can guarantee there are no mutable references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe {
            self.inner
                .as_ref()
                .unwrap()
                .wrapper()
                .audio_processor()
                .unwrap()
                .as_ref()
        }
    }

    #[inline]
    pub fn audio_processor_host_data_mut(&mut self) -> &mut <H as Host>::AudioProcessor<'_> {
        // SAFETY: we take &mut self, the only reference to the wrapper on the audio thread,
        // therefore we can guarantee there are other references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe {
            self.inner
                .as_ref()
                .unwrap()
                .wrapper()
                .audio_processor()
                .unwrap()
                .as_mut()
        }
    }

    #[inline]
    pub fn shared_plugin_handle(&mut self) -> PluginSharedHandle {
        self.inner.as_ref().unwrap().plugin_shared()
    }

    #[inline]
    pub fn audio_processor_plugin_handle(&mut self) -> PluginAudioProcessorHandle {
        PluginAudioProcessorHandle::new(self.inner.as_ref().unwrap().raw_instance().into())
    }
}

impl<H: Host> Drop for StartedPluginAudioProcessor<H> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            // SAFETY: this is called on the audio thread
            unsafe { inner.stop_processing() };
        }
    }
}

pub struct StoppedPluginAudioProcessor<H: Host> {
    pub(crate) inner: Arc<PluginInstanceInner<H>>,
    _no_sync: PhantomData<UnsafeCell<()>>,
}

impl<'a, H: 'a + Host> StoppedPluginAudioProcessor<H> {
    #[inline]
    pub(crate) fn new(inner: Arc<PluginInstanceInner<H>>) -> Self {
        Self {
            inner,
            _no_sync: PhantomData,
        }
    }

    #[inline]
    pub fn start_processing(
        self,
    ) -> Result<StartedPluginAudioProcessor<H>, ProcessingStartError<H>> {
        // SAFETY: this is called on the audio thread
        match unsafe { self.inner.start_processing() } {
            Ok(()) => Ok(StartedPluginAudioProcessor {
                inner: Some(self.inner),
                _no_sync: PhantomData,
            }),
            Err(_) => Err(ProcessingStartError { processor: self }),
        }
    }

    #[inline]
    pub fn shared_host_data(&self) -> &<H as Host>::Shared<'_> {
        self.inner.wrapper().shared()
    }

    #[inline]
    pub fn audio_processor_host_data(&self) -> &<H as Host>::AudioProcessor<'_> {
        // SAFETY: we take &self, the only reference to the wrapper on the audio thread, therefore
        // we can guarantee there are no mutable references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.inner.wrapper().audio_processor().unwrap().as_ref() }
    }

    #[inline]
    pub fn audio_processor_host_data_mut(&mut self) -> &mut <H as Host>::AudioProcessor<'_> {
        // SAFETY: we take &mut self, the only reference to the wrapper on the audio thread,
        // therefore we can guarantee there are other references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.inner.wrapper().audio_processor().unwrap().as_mut() }
    }

    #[inline]
    pub fn shared_plugin_data(&mut self) -> PluginSharedHandle {
        self.inner.plugin_shared()
    }

    #[inline]
    pub fn audio_processor_plugin_data(&mut self) -> PluginAudioProcessorHandle {
        PluginAudioProcessorHandle::new(self.inner.raw_instance().into())
    }
}

pub struct ProcessingStartError<H: Host> {
    processor: StoppedPluginAudioProcessor<H>,
}

impl<H: Host> ProcessingStartError<H> {
    #[inline]
    pub fn into_stopped_processor(self) -> StoppedPluginAudioProcessor<H> {
        self.processor
    }
}

impl<H: Host> Debug for ProcessingStartError<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to start plugin processing")
    }
}

impl<H: Host> Display for ProcessingStartError<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to start plugin processing")
    }
}

impl<H: Host> Error for ProcessingStartError<H> {}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_any!(StartedPluginAudioProcessor<()>: Sync);
    sa::assert_not_impl_any!(StoppedPluginAudioProcessor<()>: Sync);
}
