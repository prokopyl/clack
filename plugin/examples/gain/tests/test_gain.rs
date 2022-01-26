use clack_host::host::SharedHoster;
use clack_host::instance::PluginInstance;
use clack_host::{
    entry::PluginEntry,
    events::{event_types::NoteEvent, Event, EventList, TimestampedEvent},
    host::{AudioProcessorHoster, HostInfo, PluginHost, PluginHoster},
    instance::processor::audio::{AudioBuffer, AudioPorts, ChannelBuffer},
    instance::PluginAudioConfiguration,
};

use gain::clap_plugin_entry;

struct TestHost;
struct TestHostAudioProcessor;
struct TestHostShared;

impl<'a> PluginHoster<'a> for TestHost {
    type AudioProcessor = TestHostAudioProcessor;
    type Shared = TestHostShared;
}

impl AudioProcessorHoster for TestHostAudioProcessor {}
impl SharedHoster for TestHostShared {
    fn request_restart(&self) {
        todo!()
    }

    fn request_process(&self) {
        todo!()
    }

    fn request_callback(&self) {
        todo!()
    }
}

#[test]
pub fn it_works() {
    // Initialize host with basic info
    let host = PluginHost::new(HostInfo::new("test", "", "", "").unwrap());

    // Get plugin entry from the exported static
    // SAFETY: only called this once here
    let entry = unsafe { PluginEntry::from_descriptor(&clap_plugin_entry, "") }.unwrap();
    let desc = entry.plugin_descriptor(0).unwrap();
    assert_eq!(desc.id().unwrap().to_bytes(), b"gain");

    // Instantiate the desired plugin
    let mut plugin =
        PluginInstance::new(|| TestHostShared, |_| TestHost, &entry, b"gain", &host).unwrap();

    // Setting up some buffers
    let configuration = PluginAudioConfiguration {
        sample_rate: 44_100.0,
        frames_count_range: 32..=32,
    };

    let mut inputs_descriptors = AudioPorts::with_capacity(2, 1);
    let mut outputs_descriptors = AudioPorts::with_capacity(2, 1);

    let mut inputs = [vec![69f32; 32], vec![69f32; 32]];
    let mut outputs = [vec![0f32; 32], vec![0f32; 32]];

    let event = TimestampedEvent::new(1, Event::NoteOn(NoteEvent::new(42, -1, -1, 6.9)));
    let mut event_buffer_in = vec![event; 32];
    let mut event_buffer_out = vec![];

    let mut events_in = EventList::from_buffer(&mut event_buffer_in);
    let mut events_out = EventList::from_buffer(&mut event_buffer_out);

    let mut processor = plugin
        .activate(TestHostAudioProcessor, configuration)
        .unwrap()
        .start_processing()
        .unwrap();

    let input_channels = inputs_descriptors.with_buffers_f32([AudioBuffer {
        channels: inputs.iter_mut().map(|buf| ChannelBuffer::variable(buf)),
        latency: 0,
    }]);

    let output_channels = outputs_descriptors.with_buffers_f32([AudioBuffer {
        channels: outputs.iter_mut().map(|buf| ChannelBuffer::variable(buf)),
        latency: 0,
    }]);

    // Process
    processor.process(
        &input_channels,
        &output_channels,
        &mut events_in,
        &mut events_out,
    );

    // Check the gain was applied properly
    for channel_index in 0..1 {
        let inbuf = &inputs[channel_index];
        let outbuf = &outputs[channel_index];
        for (input, output) in inbuf.iter().zip(outbuf.iter()) {
            assert_eq!(*output, *input * 2.0)
        }
    }

    // Check velocity was changed properly
    assert_eq!(event_buffer_in.len(), event_buffer_out.len());

    for (input, output) in event_buffer_in.iter().zip(event_buffer_out.iter()) {
        let input_note = if let Some(Event::NoteOn(ev)) = input.event() {
            ev
        } else {
            panic!("Invalid event type found")
        };

        assert_eq!(
            output,
            &TimestampedEvent::new(
                input.time(),
                Event::NoteOn(NoteEvent::new(42, -1, -1, input_note.velocity() * 2.0))
            )
        )
    }

    plugin.deactivate(processor.stop_processing());
}
