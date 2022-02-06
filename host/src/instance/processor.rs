use crate::host::PluginHoster;
use crate::instance::processor::audio::AudioBuffers;
use crate::plugin::{PluginAudioProcessor, PluginShared};
use crate::wrapper::{HostError, HostWrapper};
use clack_common::events::io::{InputEvents, OutputEvents};
use clack_common::process::ProcessStatus;
use clap_sys::events::{clap_event_header, clap_event_transport, CLAP_TRANSPORT_IS_PLAYING};
use clap_sys::process::clap_process;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::sync::Arc;

pub mod audio;

pub struct StartedPluginAudioProcessor<'a, H: PluginHoster<'a>> {
    wrapper: Pin<Arc<HostWrapper<'a, H>>>,
}

impl<'a, H: PluginHoster<'a>> StartedPluginAudioProcessor<'a, H> {
    pub fn process(
        &mut self,
        audio_inputs: &AudioBuffers,
        audio_outputs: &mut AudioBuffers,
        events_input: &mut InputEvents,
        events_output: &mut OutputEvents,
    ) -> Result<ProcessStatus, HostError> {
        let min_input_sample_count = audio_inputs.min_buffer_length;
        let min_output_sample_count = audio_outputs.min_buffer_length;

        // TODO
        let transport = clap_event_transport {
            header: clap_event_header {
                size: 0,
                time: 0,
                space_id: 0,
                type_: 0,
                flags: 0,
            },
            flags: CLAP_TRANSPORT_IS_PLAYING,

            song_pos_beats: 0,
            song_pos_seconds: 0,
            tempo: 0.0,
            tempo_inc: 0.0,
            bar_start: 0,
            bar_number: 0,
            loop_start_beats: 0,
            loop_end_beats: 0,
            loop_start_seconds: 0,
            loop_end_seconds: 0,
            tsig_num: 4,
            tsig_denom: 4,
        };

        let process = clap_process {
            steady_time: 0, // TODO
            frames_count: min_input_sample_count.min(min_output_sample_count) as u32,
            transport: &transport, // TODO
            audio_inputs: audio_inputs.buffers.as_ptr(),
            audio_outputs: audio_outputs.buffers.as_mut_ptr(),
            audio_inputs_count: audio_inputs.buffers.len() as u32,
            audio_outputs_count: audio_outputs.buffers.len() as u32,
            in_events: events_input.as_raw(),
            out_events: events_output.as_raw_mut(),
        };

        let instance = self.wrapper.raw_instance();

        if let Some(do_process) = instance.process {
            ProcessStatus::from_raw(unsafe { do_process(instance, &process) })
                .ok_or(())
                .and_then(|r| r)
                .map_err(|_| HostError::ProcessingFailed)
        } else {
            Err(HostError::ProcessingFailed)
        }
    }

    #[inline]
    pub fn stop_processing(self) -> StoppedPluginAudioProcessor<'a, H> {
        // SAFETY: this is called on the audio thread
        unsafe { self.wrapper.stop_processing() };

        StoppedPluginAudioProcessor {
            wrapper: self.wrapper,
        }
    }

    #[inline]
    pub fn shared_host_data(&self) -> &H::Shared {
        self.wrapper.shared()
    }

    #[inline]
    pub fn audio_processor_host_data(&self) -> &H::AudioProcessor {
        // SAFETY: we take &self, the only reference to the wrapper on the audio thread, therefore
        // we can guarantee there are no mutable references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.wrapper.audio_processor().unwrap().as_ref() }
    }

    #[inline]
    pub fn audio_processor_host_data_mut(&mut self) -> &mut H::AudioProcessor {
        // SAFETY: we take &mut self, the only reference to the wrapper on the audio thread,
        // therefore we can guarantee there are other references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.wrapper.audio_processor().unwrap().as_mut() }
    }

    #[inline]
    pub fn shared_plugin_data(&mut self) -> PluginShared {
        PluginShared::new((self.wrapper.raw_instance() as *const _) as *mut _)
    }

    #[inline]
    pub fn audio_processor_plugin_data(&mut self) -> PluginAudioProcessor {
        PluginAudioProcessor::new((self.wrapper.raw_instance() as *const _) as *mut _)
    }
}

pub struct StoppedPluginAudioProcessor<'a, H: PluginHoster<'a>> {
    pub(crate) wrapper: Pin<Arc<HostWrapper<'a, H>>>,
}

impl<'a, H: PluginHoster<'a>> StoppedPluginAudioProcessor<'a, H> {
    #[inline]
    pub(crate) fn new(inner: Pin<Arc<HostWrapper<'a, H>>>) -> Self {
        Self { wrapper: inner }
    }

    #[inline]
    pub fn start_processing(
        self,
    ) -> Result<StartedPluginAudioProcessor<'a, H>, ProcessingStartError<'a, H>> {
        // SAFETY: this is called on the audio thread
        match unsafe { self.wrapper.start_processing() } {
            Ok(_) => Ok(StartedPluginAudioProcessor {
                wrapper: self.wrapper,
            }),
            Err(_) => Err(ProcessingStartError { processor: self }),
        }
    }

    #[inline]
    pub fn shared_host_data(&self) -> &H::Shared {
        self.wrapper.shared()
    }

    #[inline]
    pub fn audio_processor_host_data(&self) -> &H::AudioProcessor {
        // SAFETY: we take &self, the only reference to the wrapper on the audio thread, therefore
        // we can guarantee there are no mutable references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.wrapper.audio_processor().unwrap().as_ref() }
    }

    #[inline]
    pub fn audio_processor_host_data_mut(&mut self) -> &mut H::AudioProcessor {
        // SAFETY: we take &mut self, the only reference to the wrapper on the audio thread,
        // therefore we can guarantee there are other references anywhere
        // PANIC: This struct exists, therefore we are guaranteed the plugin is active
        unsafe { self.wrapper.audio_processor().unwrap().as_mut() }
    }

    #[inline]
    pub fn shared_plugin_data(&mut self) -> PluginShared {
        PluginShared::new((self.wrapper.raw_instance() as *const _) as *mut _)
    }

    #[inline]
    pub fn audio_processor_plugin_data(&mut self) -> PluginAudioProcessor {
        PluginAudioProcessor::new((self.wrapper.raw_instance() as *const _) as *mut _)
    }
}

pub struct ProcessingStartError<'a, H: PluginHoster<'a>> {
    processor: StoppedPluginAudioProcessor<'a, H>,
}

impl<'a, H: PluginHoster<'a>> ProcessingStartError<'a, H> {
    #[inline]
    pub fn into_stopped_processor(self) -> StoppedPluginAudioProcessor<'a, H> {
        self.processor
    }
}

impl<'a, H: PluginHoster<'a>> Debug for ProcessingStartError<'a, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to start plugin processing")
    }
}
