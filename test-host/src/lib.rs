use crate::host::{TestHostAudioProcessor, TestHostMainThread, TestHostShared};
use clack_host::entry::PluginEntryDescriptor;
use clack_host::events::{EventList, TimestampedEvent};
use clack_host::instance::processor::audio::{AudioBuffer, AudioPorts, ChannelBuffer};
use clack_host::instance::processor::StoppedPluginAudioProcessor;
use clack_host::instance::PluginAudioConfiguration;
use clack_host::{
    entry::PluginEntry,
    host::{HostInfo, PluginHost},
    instance::PluginInstance,
};

mod host;

pub struct TestHost<'a> {
    entry: PluginEntry<'a>,
    plugin: PluginInstance<'a, TestHostMainThread>,
    processor: Option<StoppedPluginAudioProcessor<'a, TestHostMainThread>>,

    input_buffers: [Vec<f32>; 2],
    output_buffers: [Vec<f32>; 2],

    input_events: Vec<TimestampedEvent<'static>>,
    output_events: Vec<TimestampedEvent<'static>>,
}

impl<'a> TestHost<'a> {
    pub fn instantiate(entry: &'a PluginEntryDescriptor) -> Self {
        // Initialize host with basic info
        let host = PluginHost::new(HostInfo::new("test", "", "", "").unwrap());

        // Get plugin entry from the exported static
        // SAFETY: only called this once here
        let entry = unsafe { PluginEntry::from_descriptor(entry, "") }.unwrap();
        let desc = entry.plugin_descriptor(0).unwrap();

        // Instantiate the desired plugin
        let plugin = PluginInstance::new(
            || TestHostShared,
            |_| TestHostMainThread,
            &entry,
            desc.id().unwrap().to_bytes(),
            &host,
        )
        .unwrap();

        Self {
            plugin,
            entry,
            processor: None,
            input_buffers: [vec![0f32; 32], vec![0f32; 32]],
            output_buffers: [vec![0f32; 32], vec![0f32; 32]],

            input_events: Vec::new(),
            output_events: Vec::new(),
        }
    }

    pub fn entry(&self) -> &PluginEntry {
        &self.entry
    }

    pub fn inputs(&self) -> &[Vec<f32>; 2] {
        &self.input_buffers
    }

    pub fn outputs(&self) -> &[Vec<f32>; 2] {
        &self.output_buffers
    }

    pub fn inputs_mut(&mut self) -> &mut [Vec<f32>; 2] {
        &mut self.input_buffers
    }

    pub fn outputs_mut(&mut self) -> &mut [Vec<f32>; 2] {
        &mut self.output_buffers
    }

    pub fn input_events(&self) -> &Vec<TimestampedEvent<'static>> {
        &self.input_events
    }

    pub fn output_events(&self) -> &Vec<TimestampedEvent<'static>> {
        &self.output_events
    }

    pub fn input_events_mut(&mut self) -> &mut Vec<TimestampedEvent<'static>> {
        &mut self.input_events
    }

    pub fn output_events_mut(&mut self) -> &mut Vec<TimestampedEvent<'static>> {
        &mut self.output_events
    }

    pub fn activate(&mut self) {
        // Setting up some buffers
        let configuration = PluginAudioConfiguration {
            sample_rate: 44_100.0,
            frames_count_range: 32..=32,
        };

        let processor = self
            .plugin
            .activate(TestHostAudioProcessor, configuration)
            .unwrap();

        self.processor = Some(processor)
    }

    pub fn process(&mut self) {
        let mut processor = self.processor.take().unwrap().start_processing().unwrap();

        let mut inputs_descriptors = AudioPorts::with_capacity(2, 1);
        let mut outputs_descriptors = AudioPorts::with_capacity(2, 1);
        let input_channels = inputs_descriptors.with_buffers_f32([AudioBuffer {
            channels: self
                .input_buffers
                .iter_mut()
                .map(|buf| ChannelBuffer::variable(buf)),
            latency: 0,
        }]);

        let output_channels = outputs_descriptors.with_buffers_f32([AudioBuffer {
            channels: self
                .output_buffers
                .iter_mut()
                .map(|buf| ChannelBuffer::variable(buf)),
            latency: 0,
        }]);

        let mut events_in = EventList::from_buffer(&mut self.input_events);
        let mut events_out = EventList::from_buffer(&mut self.output_events);

        processor.process(
            &input_channels,
            &output_channels,
            &mut events_in,
            &mut events_out,
        );

        self.processor = Some(processor.stop_processing());
    }

    pub fn deactivate(&mut self) {
        self.plugin.deactivate(self.processor.take().unwrap());
    }
}
