use clack_extensions::audio_ports::{AudioPortInfoBuffer, PluginAudioPorts};
use clack_plugin::events::event_types::ParamValueEvent;
use clack_plugin::events::{EventFlags, EventHeader};
use clack_plugin::prelude::Pckn;
use clack_plugin::utils::Cookie;
use clack_test_host::TestHost;

use clack_plugin_gain::clap_entry;

#[test]
pub fn it_works() {
    // Initialize host
    let mut host = TestHost::instantiate(&clap_entry);
    assert_eq!(
        host.descriptor().id().unwrap().to_bytes(),
        b"org.rust-audio.clack.gain"
    );
    assert_eq!(
        host.descriptor().name().unwrap().to_bytes(),
        b"Clack Gain Example"
    );

    assert!(host.descriptor().vendor().is_none());
    assert!(host.descriptor().url().is_none());
    assert!(host.descriptor().manual_url().is_none());
    assert!(host.descriptor().support_url().is_none());
    assert!(host.descriptor().description().is_none());
    assert!(host.descriptor().version().is_none());

    assert_eq!(
        host.descriptor()
            .features()
            .map(|s| s.to_bytes())
            .collect::<Vec<_>>(),
        &[&b"audio-effect"[..], &b"stereo"[..]]
    );

    let plugin = host.plugin_mut();

    let mut plugin_main_thread = plugin.plugin_handle();
    let ports_ext = plugin_main_thread
        .get_extension::<PluginAudioPorts>()
        .unwrap();
    assert_eq!(1, ports_ext.count(&mut plugin_main_thread, true));
    assert_eq!(1, ports_ext.count(&mut plugin_main_thread, false));

    let mut buf = AudioPortInfoBuffer::new();
    let info = ports_ext
        .get(&mut plugin_main_thread, 0, false, &mut buf)
        .unwrap();

    assert_eq!(info.id, 0);
    assert_eq!(info.name, b"main");

    host.activate();

    assert!(host.plugin().is_active());

    host.input_events_mut().push(&ParamValueEvent::new(
        EventHeader::new_core(0, EventFlags::empty()),
        1,
        Pckn::match_all(),
        0.5,
        Cookie::empty(),
    ));

    host.inputs_mut()[0].fill(69f32);
    host.inputs_mut()[1].fill(69f32);

    host.process().unwrap();

    // Check the gain was applied properly
    for channel_index in 0..1 {
        let inbuf = &host.inputs()[channel_index];
        let outbuf = &host.outputs()[channel_index];
        for (input, output) in inbuf.iter().zip(outbuf.iter()) {
            assert_eq!(*output, *input * 0.5)
        }
    }

    host.deactivate();
}
