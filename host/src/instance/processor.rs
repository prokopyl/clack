use crate::extensions::wrapper::instance::PluginInstanceInner;
use crate::host::Host;
use crate::host::HostError;
use crate::instance::handle::{PluginAudioProcessorHandle, PluginSharedHandle};
use crate::instance::processor::audio::InputAudioBuffers;
use crate::instance::processor::PluginAudioProcessor::*;
use crate::prelude::OutputAudioBuffers;
use clack_common::events::event_types::TransportEvent;
use clack_common::events::io::{InputEvents, OutputEvents};
use clack_common::process::ProcessStatus;
use clap_sys::process::clap_process;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

pub mod audio;

pub enum PluginAudioProcessor<H: for<'a> Host<'a>> {
    Started(StartedPluginAudioProcessor<H>),
    Stopped(StoppedPluginAudioProcessor<H>),
    Poisoned,
}

impl<'a, H: 'a + for<'h> Host<'h>> PluginAudioProcessor<H> {
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
    pub fn shared_host_data(&self) -> &<H as Host>::Shared {
        match self {
            Poisoned => panic!("Plugin audio processor was poisoned"),
            Started(s) => s.shared_host_data(),
            Stopped(s) => s.shared_host_data(),
        }
    }

    #[inline]
    pub fn audio_processor_host_data(&self) -> &<H as Host>::AudioProcessor {
        match self {
            Poisoned => panic!("Plugin audio processor was poisoned"),
            Started(s) => s.audio_processor_host_data(),
            Stopped(s) => s.audio_processor_host_data(),
        }
    }

    #[inline]
    pub fn audio_processor_host_data_mut(&mut self) -> &mut <H as Host>::AudioProcessor {
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

impl<'a, H: 'a + for<'h> Host<'h>> From<StartedPluginAudioProcessor<H>>
    for PluginAudioProcessor<H>
{
    #[inline]
    fn from(p: StartedPluginAudioProcessor<H>) -> Self {
        Started(p)
    }
}

impl<'a, H: 'a + for<'h> Host<'h>> From<StoppedPluginAudioProcessor<H>>
    for PluginAudioProcessor<H>
{
    #[inline]
    fn from(p: StoppedPluginAudioProcessor<H>) -> Self {
        Stopped(p)
    }
}

pub struct StartedPluginAudioProcessor<H: for<'a> Host<'a>> {
    inner: Arc<PluginInstanceInner<H>>,
}

impl<H: for<'h> Host<'h>> StartedPluginAudioProcessor<H> {
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
        let min_input_sample_count = audio_inputs.min_buffer_length;
        let min_output_sample_count = audio_outputs.min_buffer_length;

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
            audio_inputs: audio_inputs.buffers.as_ptr(),
            audio_outputs: audio_outputs.output_data.as_mut_ptr(),
            audio_inputs_count: audio_inputs.buffers.len() as u32,
            audio_outputs_count: audio_outputs.output_data.len() as u32,
            in_events: events_input.as_raw(),
            out_events: events_output.as_raw_mut() as *mut _,
        };

        let instance = self.inner.raw_instance();

        let status = ProcessStatus::from_raw(unsafe {
            (instance.process.ok_or(HostError::NullProcessFunction)?)(instance, &process)
        })
        .ok_or(())
        .and_then(|r| r)
        .map_err(|_| HostError::ProcessingFailed)?;

        Ok(status)
    }

    #[inline]
    pub fn stop_processing(self) -> StoppedPluginAudioProcessor<H> {
        // SAFETY: this is called on the audio thread
        unsafe { self.inner.stop_processing() };

        StoppedPluginAudioProcessor { inner: self.inner }
    }

    #[inline]
    pub fn shared_host_data(&self) -> &<H as Host>::Shared {
        self.inner.wrapper().shared()
    }

    #[inline]
    pub fn audio_processor_host_data(&self) -> &<H as Host>::AudioProcessor {
        // SAFETY: we take &self, the only reference to the wrapper on the audio thread, therefore
        // we can guarantee there are no mutable references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.inner.wrapper().audio_processor().unwrap().as_ref() }
    }

    #[inline]
    pub fn audio_processor_host_data_mut(&mut self) -> &mut <H as Host>::AudioProcessor {
        // SAFETY: we take &mut self, the only reference to the wrapper on the audio thread,
        // therefore we can guarantee there are other references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.inner.wrapper().audio_processor().unwrap().as_mut() }
    }

    #[inline]
    pub fn shared_plugin_handle(&mut self) -> PluginSharedHandle {
        PluginSharedHandle::new((self.inner.raw_instance() as *const _) as *mut _)
    }

    #[inline]
    pub fn audio_processor_plugin_handle(&mut self) -> PluginAudioProcessorHandle {
        PluginAudioProcessorHandle::new((self.inner.raw_instance() as *const _) as *mut _)
    }
}

pub struct StoppedPluginAudioProcessor<H: for<'a> Host<'a>> {
    pub(crate) inner: Arc<PluginInstanceInner<H>>,
}

impl<'a, H: 'a + for<'h> Host<'h>> StoppedPluginAudioProcessor<H> {
    #[inline]
    pub(crate) fn new(inner: Arc<PluginInstanceInner<H>>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn start_processing(
        self,
    ) -> Result<StartedPluginAudioProcessor<H>, ProcessingStartError<H>> {
        // SAFETY: this is called on the audio thread
        match unsafe { self.inner.start_processing() } {
            Ok(()) => Ok(StartedPluginAudioProcessor { inner: self.inner }),
            Err(_) => Err(ProcessingStartError { processor: self }),
        }
    }

    #[inline]
    pub fn shared_host_data(&self) -> &<H as Host>::Shared {
        self.inner.wrapper().shared()
    }

    #[inline]
    pub fn audio_processor_host_data(&self) -> &<H as Host>::AudioProcessor {
        // SAFETY: we take &self, the only reference to the wrapper on the audio thread, therefore
        // we can guarantee there are no mutable references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.inner.wrapper().audio_processor().unwrap().as_ref() }
    }

    #[inline]
    pub fn audio_processor_host_data_mut(&mut self) -> &mut <H as Host>::AudioProcessor {
        // SAFETY: we take &mut self, the only reference to the wrapper on the audio thread,
        // therefore we can guarantee there are other references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.inner.wrapper().audio_processor().unwrap().as_mut() }
    }

    #[inline]
    pub fn shared_plugin_data(&mut self) -> PluginSharedHandle {
        PluginSharedHandle::new((self.inner.raw_instance() as *const _) as *mut _)
    }

    #[inline]
    pub fn audio_processor_plugin_data(&mut self) -> PluginAudioProcessorHandle {
        PluginAudioProcessorHandle::new((self.inner.raw_instance() as *const _) as *mut _)
    }
}

pub struct ProcessingStartError<H: for<'a> Host<'a>> {
    processor: StoppedPluginAudioProcessor<H>,
}

impl<H: for<'h> Host<'h>> ProcessingStartError<H> {
    #[inline]
    pub fn into_stopped_processor(self) -> StoppedPluginAudioProcessor<H> {
        self.processor
    }
}

impl<H: for<'h> Host<'h>> Debug for ProcessingStartError<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to start plugin processing")
    }
}

impl<H: for<'h> Host<'h>> Display for ProcessingStartError<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to start plugin processing")
    }
}

impl<H: for<'h> Host<'h>> Error for ProcessingStartError<H> {}
