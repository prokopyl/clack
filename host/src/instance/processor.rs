use crate::instance::channel::PluginInstanceChannelSend;
use crate::instance::processor::audio::HostAudioBufferCollection;
use crate::instance::processor::inner::PluginAudioProcessorInner;
use clap_audio_common::events::list::EventList;
use clap_sys::events::{clap_event_transport, CLAP_TRANSPORT_IS_PLAYING};
use clap_sys::process::clap_process;
use std::fmt::{Debug, Formatter};

pub(crate) mod inner;

pub mod audio;

pub enum PluginAudioProcessor<TChannel: PluginInstanceChannelSend> {
    Started(StartedPluginAudioProcessor<TChannel>),
    Stopped(StoppedPluginAudioProcessor<TChannel>),
}

impl<TChannel: PluginInstanceChannelSend> From<StartedPluginAudioProcessor<TChannel>>
    for PluginAudioProcessor<TChannel>
{
    #[inline]
    fn from(processor: StartedPluginAudioProcessor<TChannel>) -> Self {
        PluginAudioProcessor::Started(processor)
    }
}

impl<TChannel: PluginInstanceChannelSend> From<StoppedPluginAudioProcessor<TChannel>>
    for PluginAudioProcessor<TChannel>
{
    #[inline]
    fn from(processor: StoppedPluginAudioProcessor<TChannel>) -> Self {
        PluginAudioProcessor::Stopped(processor)
    }
}

pub struct StartedPluginAudioProcessor<TChannel: PluginInstanceChannelSend> {
    inner: PluginAudioProcessorInner<TChannel>,
}

impl<TChannel: PluginInstanceChannelSend> StartedPluginAudioProcessor<TChannel> {
    pub fn process<B, S>(
        &mut self,
        audio_inputs: &HostAudioBufferCollection<B, S>,
        audio_outputs: &mut HostAudioBufferCollection<B, S>,
        events_input: &mut EventList,
        events_output: &mut EventList,
    ) {
        let min_input_sample_count = audio_inputs.min_buffer_length();
        let min_output_sample_count = audio_outputs.min_buffer_length();

        // TODO
        let transport = clap_event_transport {
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
            audio_inputs: audio_inputs.raw_buffers(),
            audio_outputs: audio_outputs.raw_buffers(),
            audio_inputs_count: audio_inputs.port_count() as u32,
            audio_outputs_count: audio_outputs.port_count() as u32,
            in_events: events_input.as_raw_mut(),
            out_events: events_output.as_raw_mut(),
        };

        unsafe { self.inner.process(&process) };
    }

    #[inline]
    pub fn stop_processing(mut self) -> StoppedPluginAudioProcessor<TChannel> {
        unsafe { self.inner.stop_processing() };

        StoppedPluginAudioProcessor { inner: self.inner }
    }
}

// TODO: unsound if the entry (i.e. the dyn lib file) gets dropped first
pub struct StoppedPluginAudioProcessor<TChannel: PluginInstanceChannelSend> {
    inner: PluginAudioProcessorInner<TChannel>, // TODO: accessors
}

impl<TChannel: PluginInstanceChannelSend> StoppedPluginAudioProcessor<TChannel> {
    #[inline]
    pub(crate) fn new(inner: PluginAudioProcessorInner<TChannel>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn start_processing(mut self) -> Result<StartedPluginAudioProcessor<TChannel>, Self> {
        let success = unsafe { self.inner.start_processing() };

        match success {
            true => Ok(StartedPluginAudioProcessor { inner: self.inner }),
            false => Err(self),
        }
    }
}

impl<TChannel: PluginInstanceChannelSend> Debug for StoppedPluginAudioProcessor<TChannel> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to start plugin processing")
    }
}
