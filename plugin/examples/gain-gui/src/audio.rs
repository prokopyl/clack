//! Contains all types and implementations related to audio processing and the audio thread.

use crate::params::GainParamsShared;
use crate::params::{GainParamsLocal, GestureChange};
use crate::{GainPluginMainThread, GainPluginShared};
use clack_extensions::audio_ports::*;
use clack_extensions::params::PluginAudioProcessorParams;
use clack_plugin::events::event_types::{ParamGestureBeginEvent, ParamGestureEndEvent};
use clack_plugin::prelude::*;

/// Our plugin's audio processor. It lives in the audio thread.
///
/// It receives parameter events, and process a stereo audio signal by operating on the given audio
/// buffer.
pub struct GainPluginAudioProcessor<'a> {
    /// The local state of the parameters
    params: GainParamsLocal,
    /// A reference to the plugin's shared data.
    shared: &'a GainPluginShared,
    /// Our handle to the host
    host: HostAudioProcessorHandle<'a>,
}

impl<'a> PluginAudioProcessor<'a, GainPluginShared, GainPluginMainThread<'a>>
    for GainPluginAudioProcessor<'a>
{
    fn activate(
        host: HostAudioProcessorHandle<'a>,
        _main_thread: &mut GainPluginMainThread,
        shared: &'a GainPluginShared,
        _audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        // This is where we would allocate intermediate buffers and such if we needed them.
        Ok(Self {
            host,
            shared,
            params: GainParamsLocal::new(&shared.params),
        })
    }

    fn process(
        &mut self,
        _process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        // First, we have to make a few sanity checks.
        // We want at least a single input/output port pair, which contains channels of `f32`
        // audio sample data.
        let mut port_pair = audio
            .port_pair(0)
            .ok_or(PluginError::Message("No input/output ports found"))?;

        let mut output_channels = port_pair
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Expected f32 input/output"))?;

        let mut channel_buffers = [None, None];

        // Extract the buffer slices that we need, while making sure they are paired correctly and
        // check for either in-place or separate buffers.
        for (pair, buf) in output_channels.iter_mut().zip(&mut channel_buffers) {
            *buf = match pair {
                ChannelPair::InputOnly(_) => None,
                ChannelPair::OutputOnly(_) => None,
                ChannelPair::InPlace(b) => Some(b),
                ChannelPair::InputOutput(i, o) => {
                    o.copy_from_slice(i);
                    Some(o)
                }
            }
        }

        // Receive any param updates from the main thread and/or the GUI.
        let has_ui_param_updates = self.params.fetch_updates(&self.shared.params);

        // Now let's process the audio, while splitting the processing in batches between each
        // sample-accurate event.

        for event_batch in events.input.batch() {
            // Process all param events in this batch
            for event in event_batch.events() {
                self.params.handle_event(event)
            }

            // Get the volume value after all parameter changes have been handled.
            let volume = self.params.get_volume();

            for buf in channel_buffers.iter_mut().flatten() {
                for sample in buf.iter_mut() {
                    *sample *= volume
                }
            }
        }

        // Publish any parameter changes we may have received back to the GUI.
        if self.params.push_updates(&self.shared.params) {
            // Request the on-main-thread callback, which we use to refresh the UI if it is open
            self.host.request_callback();
        }

        // Fetch the latest gesture status
        let current_gesture = self
            .params
            .fetch_gesture(&self.shared.params, has_ui_param_updates);

        // Send a Gesture Begin event, if we need to do so
        if let Some(GestureChange::Begin | GestureChange::Both) = current_gesture {
            let _ = events.output.try_push(ParamGestureBeginEvent::new(
                0,
                GainParamsShared::PARAM_VOLUME_ID,
            ));
        }

        // If the UI sent us param updates, send them to the Host
        if has_ui_param_updates {
            self.params.send_param_events(events.output);
        }

        // Send a Gesture End event, if we need to do so
        if let Some(GestureChange::End | GestureChange::Both) = current_gesture {
            let _ = events.output.try_push(ParamGestureEndEvent::new(
                audio.frames_count(),
                GainParamsShared::PARAM_VOLUME_ID,
            ));
        }

        Ok(ProcessStatus::ContinueIfNotQuiet)
    }
}

impl PluginAudioPortsImpl for GainPluginMainThread<'_> {
    fn count(&mut self, _is_input: bool) -> u32 {
        1
    }

    fn get(&mut self, index: u32, _is_input: bool, writer: &mut AudioPortInfoWriter) {
        if index == 0 {
            writer.set(&AudioPortInfo {
                id: ClapId::new(0),
                name: b"main",
                channel_count: 2,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::STEREO),
                in_place_pair: None,
            });
        }
    }
}

impl PluginAudioProcessorParams for GainPluginAudioProcessor<'_> {
    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut OutputEvents,
    ) {
        for event in input_parameter_changes {
            self.params.handle_event(event)
        }
    }
}
