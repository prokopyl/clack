use crate::host::Host;
use crate::host::HostError;
use crate::instance::processor::audio::AudioBuffers;
use crate::instance::processor::PluginAudioProcessorState::*;
use crate::plugin::{PluginAudioProcessorHandle, PluginSharedHandle};
use crate::wrapper::instance::PluginInstanceInner;
use clack_common::events::io::{InputEvents, OutputEvents};
use clack_common::process::ProcessStatus;
use clap_sys::process::clap_process;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub mod audio;

pub enum PluginAudioProcessorState<H: for<'a> Host<'a>> {
    Started(StartedPluginAudioProcessor<H>),
    Stopped(StoppedPluginAudioProcessor<H>),
}

// TODO: bikeshed a lot
pub struct PluginAudioProcessor<H: for<'a> Host<'a>> {
    poisonable_inner: Option<PluginAudioProcessorState<H>>,
}

impl<'a, H: 'a + for<'h> Host<'h>> PluginAudioProcessor<H> {
    pub fn as_started(&self) -> Result<&StartedPluginAudioProcessor<H>, HostError> {
        match self
            .poisonable_inner
            .as_ref()
            .ok_or(HostError::ProcessorHandlePoisoned)?
        {
            Started(s) => Ok(s),
            Stopped(_) => Err(HostError::ProcessingStopped),
        }
    }

    pub fn as_started_mut(&mut self) -> Result<&mut StartedPluginAudioProcessor<H>, HostError> {
        match self
            .poisonable_inner
            .as_mut()
            .ok_or(HostError::ProcessorHandlePoisoned)?
        {
            Started(s) => Ok(s),
            Stopped(_) => Err(HostError::ProcessingStopped),
        }
    }

    pub fn as_stopped(&self) -> Result<&StoppedPluginAudioProcessor<H>, HostError> {
        match self
            .poisonable_inner
            .as_ref()
            .ok_or(HostError::ProcessorHandlePoisoned)?
        {
            Stopped(s) => Ok(s),
            Started(_) => Err(HostError::ProcessingStarted),
        }
    }

    pub fn as_stopped_mut(&mut self) -> Result<&mut StoppedPluginAudioProcessor<H>, HostError> {
        match self
            .poisonable_inner
            .as_mut()
            .ok_or(HostError::ProcessorHandlePoisoned)?
        {
            Stopped(s) => Ok(s),
            Started(_) => Err(HostError::ProcessingStarted),
        }
    }

    #[inline]
    pub fn shared_host_data(&self) -> &<H as Host>::Shared {
        match self.poisonable_inner.as_ref().unwrap() {
            Started(s) => s.shared_host_data(),
            Stopped(s) => s.shared_host_data(),
        }
    }

    #[inline]
    pub fn audio_processor_host_data(&self) -> &<H as Host>::AudioProcessor {
        match self.poisonable_inner.as_ref().unwrap() {
            Started(s) => s.audio_processor_host_data(),
            Stopped(s) => s.audio_processor_host_data(),
        }
    }

    #[inline]
    pub fn audio_processor_host_data_mut(&mut self) -> &mut <H as Host>::AudioProcessor {
        match self.poisonable_inner.as_mut().unwrap() {
            Started(s) => s.audio_processor_host_data_mut(),
            Stopped(s) => s.audio_processor_host_data_mut(),
        }
    }

    pub fn is_started(&self) -> bool {
        match self.poisonable_inner.as_ref() {
            None => false,
            Some(Started(_)) => true,
            Some(Stopped(_)) => false,
        }
    }

    pub fn start_processing(&mut self) -> Result<(), HostError> {
        let inner = self
            .poisonable_inner
            .take()
            .ok_or(HostError::ProcessorHandlePoisoned)?;

        match inner {
            Started(s) => {
                self.poisonable_inner = Some(Started(s));
                Err(HostError::ProcessingStarted)
            }
            Stopped(s) => match s.start_processing() {
                Ok(s) => {
                    self.poisonable_inner = Some(Started(s));
                    Ok(())
                }
                Err(e) => {
                    self.poisonable_inner = Some(Stopped(e.processor));
                    Err(HostError::StartProcessingFailed)
                }
            },
        }
    }

    pub fn stop_processing(&mut self) -> Result<(), HostError> {
        let inner = self
            .poisonable_inner
            .take()
            .ok_or(HostError::ProcessorHandlePoisoned)?;

        match inner {
            Stopped(s) => {
                self.poisonable_inner = Some(Stopped(s));
                Err(HostError::ProcessingStopped)
            }
            Started(s) => {
                self.poisonable_inner = Some(Stopped(s.stop_processing()));
                Ok(())
            }
        }
    }
}

impl<'a, H: 'a + for<'h> Host<'h>> From<StartedPluginAudioProcessor<H>>
    for PluginAudioProcessor<H>
{
    #[inline]
    fn from(p: StartedPluginAudioProcessor<H>) -> Self {
        Self {
            poisonable_inner: Some(Started(p)),
        }
    }
}

impl<'a, H: 'a + for<'h> Host<'h>> From<StoppedPluginAudioProcessor<H>>
    for PluginAudioProcessor<H>
{
    #[inline]
    fn from(p: StoppedPluginAudioProcessor<H>) -> Self {
        Self {
            poisonable_inner: Some(Stopped(p)),
        }
    }
}

pub struct StartedPluginAudioProcessor<H: for<'a> Host<'a>> {
    inner: Arc<PluginInstanceInner<H>>,
}

impl<'a, H: 'a + for<'h> Host<'h>> StartedPluginAudioProcessor<H> {
    pub fn process(
        &mut self,
        audio_inputs: &AudioBuffers,
        audio_outputs: &mut AudioBuffers,
        events_input: &mut InputEvents,
        events_output: &mut OutputEvents,
        steady_time: i64,
        max_frame_count: Option<usize>,
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
            transport: core::ptr::null(), // TODO
            audio_inputs: audio_inputs.buffers.as_ptr(),
            audio_outputs: audio_outputs.buffers.as_mut_ptr(),
            audio_inputs_count: audio_inputs.buffers.len() as u32,
            audio_outputs_count: audio_outputs.buffers.len() as u32,
            in_events: events_input.as_raw(),
            out_events: events_output.as_raw_mut(),
        };

        let instance = self.inner.raw_instance();

        ProcessStatus::from_raw(unsafe { (instance.process)(instance, &process) })
            .ok_or(())
            .and_then(|r| r)
            .map_err(|_| HostError::ProcessingFailed)
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

impl<'a, H: 'a + for<'h> Host<'h>> ProcessingStartError<H> {
    #[inline]
    pub fn into_stopped_processor(self) -> StoppedPluginAudioProcessor<H> {
        self.processor
    }
}

impl<'a, H: 'a + for<'h> Host<'h>> Debug for ProcessingStartError<H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to start plugin processing")
    }
}
