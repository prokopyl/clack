use clack_plugin::events::event_types::NoteEvent;
use clack_plugin::prelude::*;
use clack_test_host::TestHost;

use gain::clap_plugin_entry;

#[test]
pub fn it_works() {
    // Initialize host
    let mut host = TestHost::instantiate(&clap_plugin_entry);
    assert_eq!(
        host.entry()
            .plugin_descriptor(0)
            .unwrap()
            .id()
            .unwrap()
            .to_bytes(),
        b"gain"
    );
    host.activate();

    host.inputs_mut()[0].fill(69f32);
    host.inputs_mut()[1].fill(69f32);

    let event = TimestampedEvent::new(1, Event::NoteOn(NoteEvent::new(42, -1, -1, 6.9)));
    *host.input_events_mut() = vec![event; 32];

    host.process();

    // Check the gain was applied properly
    for channel_index in 0..1 {
        let inbuf = &host.inputs()[channel_index];
        let outbuf = &host.outputs()[channel_index];
        for (input, output) in inbuf.iter().zip(outbuf.iter()) {
            assert_eq!(*output, *input * 2.0)
        }
    }

    // Check velocity was changed properly
    assert_eq!(host.input_events().len(), host.output_events().len());

    for (input, output) in host.input_events().iter().zip(host.output_events().iter()) {
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

    host.deactivate();
}
