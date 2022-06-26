use crate::host::{TestHostAudioProcessor, TestHostMainThread, TestHostShared};
use clack_host::entry::{PluginDescriptor, PluginEntryDescriptor};
use clack_host::events::io::{EventBuffer, InputEvents, OutputEvents};
use clack_host::factory::PluginFactory;
use clack_host::instance::processor::audio::{
    AudioPortBuffer, AudioPortBufferType, AudioPorts, ChannelBuffer,
};
use clack_host::instance::processor::StoppedPluginAudioProcessor;
use clack_host::instance::PluginAudioConfiguration;
use clack_host::process::ProcessStatus;
use clack_host::wrapper::HostError;
use clack_host::{
    entry::PluginEntry,
    host::{HostInfo, PluginHost},
    instance::PluginInstance,
};
use std::vec::IntoIter;

mod host;

pub struct TestHost<'a> {
    entry: PluginEntry<'a>,
    plugin: PluginInstance<'a, TestHostMainThread>,
    descriptor: PluginDescriptor<'a>,
    processor: Option<StoppedPluginAudioProcessor<'a, TestHostMainThread>>,

    input_buffers: [Vec<f32>; 2],
    output_buffers: [Vec<f32>; 2],

    input_events: EventBuffer,
    output_events: EventBuffer,
}

impl<'a> TestHost<'a> {
    pub fn instantiate(entry: &'a PluginEntryDescriptor) -> Self {
        // Initialize host with basic info
        let host = PluginHost::new(HostInfo::new("test", "", "", "").unwrap());

        // Get plugin entry from the exported static
        // SAFETY: only called this once here
        let entry = unsafe { PluginEntry::from_raw(entry, "") }.unwrap();
        let descriptor = entry
            .get_factory::<PluginFactory>()
            .unwrap()
            .plugin_descriptor(0)
            .unwrap();

        // Instantiate the desired plugin
        let plugin = PluginInstance::new(
            || TestHostShared,
            |_| TestHostMainThread,
            &entry,
            descriptor.id().unwrap().to_bytes(),
            &host,
        )
        .unwrap();

        Self {
            plugin,
            entry,
            descriptor,
            processor: None,
            input_buffers: [vec![0f32; 32], vec![0f32; 32]],
            output_buffers: [vec![0f32; 32], vec![0f32; 32]],

            input_events: EventBuffer::with_capacity(10),
            output_events: EventBuffer::with_capacity(10),
        }
    }

    pub fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
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

    pub fn input_events(&self) -> &EventBuffer {
        &self.input_events
    }

    pub fn output_events(&self) -> &EventBuffer {
        &self.output_events
    }

    pub fn input_events_mut(&mut self) -> &mut EventBuffer {
        &mut self.input_events
    }

    pub fn output_events_mut(&mut self) -> &mut EventBuffer {
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

    pub fn process(&mut self) -> Result<ProcessStatus, HostError> {
        let mut processor = self.processor.take().unwrap().start_processing().unwrap();

        let mut inputs_descriptors = AudioPorts::with_capacity(2, 1);
        let mut outputs_descriptors = AudioPorts::with_capacity(2, 1);

        let input_channels = inputs_descriptors
            .with_data::<_, _, _, IntoIter<ChannelBuffer<_>>, _, &mut [f64]>([AudioPortBuffer {
                channels: AudioPortBufferType::F32(
                    self.input_buffers
                        .iter_mut()
                        .map(|buf| ChannelBuffer::variable(buf.as_mut_slice())),
                ),
                latency: 0,
            }]);

        let mut output_channels = outputs_descriptors
            .with_data::<_, _, _, IntoIter<ChannelBuffer<_>>, _, &mut [f64]>([AudioPortBuffer {
                channels: AudioPortBufferType::F32(
                    self.output_buffers
                        .iter_mut()
                        .map(|buf| ChannelBuffer::variable(buf.as_mut_slice())),
                ),
                latency: 0,
            }]);

        let mut events_in = InputEvents::from_buffer(&self.input_events);
        let mut events_out = OutputEvents::from_buffer(&mut self.output_events);

        let result = processor.process(
            &input_channels,
            &mut output_channels,
            &mut events_in,
            &mut events_out,
            0,
            None,
        );

        self.processor = Some(processor.stop_processing());

        result
    }

    #[inline]
    pub fn plugin(&self) -> &PluginInstance<'a, TestHostMainThread> {
        &self.plugin
    }

    pub fn deactivate(&mut self) {
        self.plugin.deactivate(self.processor.take().unwrap());
    }
}
