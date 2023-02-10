use crate::host::{TestHostAudioProcessor, TestHostImpl, TestHostMainThread, TestHostShared};
use clack_host::bundle::{PluginBundle, PluginEntryDescriptor};
use clack_host::events::io::{EventBuffer, InputEvents, OutputEvents};
use clack_host::factory::{PluginDescriptor, PluginFactory};
use clack_host::host::HostError;
use clack_host::instance::processor::audio::{
    AudioPortBuffer, AudioPortBufferType, AudioPorts, InputChannel,
};
use clack_host::instance::processor::StoppedPluginAudioProcessor;
use clack_host::instance::PluginAudioConfiguration;
use clack_host::process::ProcessStatus;
use clack_host::{host::HostInfo, instance::PluginInstance};
use selfie::refs::RefType;
use selfie::Selfie;

mod host;

struct PluginDescriptorRef;

impl<'a> RefType<'a> for PluginDescriptorRef {
    type Ref = PluginDescriptor<'a>;
}

pub struct TestHost {
    entry_and_descriptor: Selfie<'static, Box<PluginBundle>, PluginDescriptorRef>,
    plugin: PluginInstance<TestHostImpl>,
    processor: Option<StoppedPluginAudioProcessor<TestHostImpl>>,

    input_buffers: [Vec<f32>; 2],
    output_buffers: [Vec<f32>; 2],

    input_events: EventBuffer,
    output_events: EventBuffer,
}

impl TestHost {
    pub fn instantiate(entry: &'static PluginEntryDescriptor) -> Self {
        // Initialize host with basic info
        let info = HostInfo::new("test", "", "", "").unwrap();

        // Get plugin entry from the exported static
        // SAFETY: only called this once here
        let entry = unsafe { PluginBundle::load_from_raw(entry, "") }.unwrap();

        let entry_and_descriptor: Selfie<_, PluginDescriptorRef> =
            Selfie::new(Box::pin(entry), |entry| {
                entry
                    .get_factory::<PluginFactory>()
                    .unwrap()
                    .plugin_descriptor(0)
                    .unwrap()
            });

        // Instantiate the desired plugin
        let plugin = PluginInstance::new(
            |_| TestHostShared,
            |_| TestHostMainThread,
            entry_and_descriptor.owned(),
            entry_and_descriptor.referential().id().unwrap(),
            &info,
        )
        .unwrap();

        Self {
            plugin,
            entry_and_descriptor,
            processor: None,
            input_buffers: [vec![0f32; 32], vec![0f32; 32]],
            output_buffers: [vec![0f32; 32], vec![0f32; 32]],

            input_events: EventBuffer::with_capacity(10),
            output_events: EventBuffer::with_capacity(10),
        }
    }

    pub fn descriptor(&self) -> PluginDescriptor {
        self.entry_and_descriptor.referential()
    }

    pub fn bundle(&self) -> &PluginBundle {
        self.entry_and_descriptor.owned()
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
            .activate(|_, _, _| TestHostAudioProcessor, configuration)
            .unwrap();

        self.processor = Some(processor)
    }

    pub fn process(&mut self) -> Result<ProcessStatus, HostError> {
        let mut processor = self.processor.take().unwrap().start_processing().unwrap();

        let mut inputs_descriptors = AudioPorts::with_capacity(2, 1);
        let mut outputs_descriptors = AudioPorts::with_capacity(2, 1);

        let input_channels = inputs_descriptors.with_input_buffers([AudioPortBuffer {
            channels: AudioPortBufferType::f32_input_only(
                self.input_buffers.iter_mut().map(InputChannel::variable),
            ),
            latency: 0,
        }]);

        let mut output_channels = outputs_descriptors.with_output_buffers([AudioPortBuffer {
            channels: AudioPortBufferType::f32_output_only(
                self.output_buffers.iter_mut().map(|b| b.as_mut_slice()),
            ),
            latency: 0,
        }]);

        let events_in = InputEvents::from_buffer(&self.input_events);
        let mut events_out = OutputEvents::from_buffer(&mut self.output_events);

        let result = processor.process(
            &input_channels,
            &mut output_channels,
            &events_in,
            &mut events_out,
            0,
            None,
            None,
        );

        self.processor = Some(processor.stop_processing());

        result
    }

    #[inline]
    pub fn plugin(&self) -> &PluginInstance<TestHostImpl> {
        &self.plugin
    }

    pub fn deactivate(&mut self) {
        self.plugin.deactivate(self.processor.take().unwrap());
    }
}
