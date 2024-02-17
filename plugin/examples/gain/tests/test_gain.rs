use clack_extensions::audio_ports::{AudioPortInfoBuffer, PluginAudioPorts};
use clack_test_host::TestHost;

use gain::clap_entry;

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

    assert!(host.descriptor().vendor().unwrap().to_bytes().is_empty());
    assert!(host.descriptor().url().unwrap().to_bytes().is_empty());
    assert!(host
        .descriptor()
        .manual_url()
        .unwrap()
        .to_bytes()
        .is_empty());
    assert!(host
        .descriptor()
        .support_url()
        .unwrap()
        .to_bytes()
        .is_empty());
    assert!(host
        .descriptor()
        .description()
        .unwrap()
        .to_bytes()
        .is_empty());
    assert!(host.descriptor().version().unwrap().to_bytes().is_empty());

    assert_eq!(
        host.descriptor()
            .features()
            .map(|s| s.to_bytes())
            .collect::<Vec<_>>(),
        &[&b"synthesizer"[..], &b"stereo"[..]]
    );

    let plugin = host.plugin_mut();

    let mut plugin_main_thread = plugin.main_thread_plugin_data();
    let ports_ext = plugin_main_thread
        .shared()
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

    host.inputs_mut()[0].fill(69f32);
    host.inputs_mut()[1].fill(69f32);

    host.process().unwrap();

    // Check the gain was applied properly
    for channel_index in 0..1 {
        let inbuf = &host.inputs()[channel_index];
        let outbuf = &host.outputs()[channel_index];
        for (input, output) in inbuf.iter().zip(outbuf.iter()) {
            assert_eq!(*output, *input * 2.0)
        }
    }

    host.deactivate();
}
