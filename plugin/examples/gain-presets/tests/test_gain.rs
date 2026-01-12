use clack_extensions::audio_ports::{AudioPortInfoBuffer, PluginAudioPorts};
use clack_extensions::preset_discovery::prelude::*;
use clack_host::events::event_types::ParamValueEvent;
use clack_host::factory::plugin::PluginFactory;
use clack_host::prelude::*;
use clack_host::utils::{Cookie, Timestamp, UniversalPluginId};
use clack_plugin_gain_presets::GainPluginEntry;
use std::ffi::{CStr, CString};

#[test]
pub fn it_works() {
    // Initialize host with basic info
    let info = HostInfo::new("test", "", "", "").unwrap();

    let bundle = PluginBundle::load_from_clack::<GainPluginEntry>(c"").unwrap();

    let descriptor = bundle
        .get_factory::<PluginFactory>()
        .unwrap()
        .plugin_descriptor(0)
        .unwrap();

    assert_eq!(
        descriptor.id().unwrap().to_bytes(),
        b"org.rust-audio.clack.gain-presets"
    );

    assert_eq!(descriptor.name().unwrap(), c"Clack Gain Presets Example");

    assert!(descriptor.vendor().is_none());
    assert!(descriptor.url().is_none());
    assert!(descriptor.manual_url().is_none());
    assert!(descriptor.support_url().is_none());
    assert!(descriptor.description().is_none());
    assert!(descriptor.version().is_none());

    assert_eq!(
        descriptor
            .features()
            .map(|s| s.to_bytes())
            .collect::<Vec<_>>(),
        &[&b"audio-effect"[..], &b"stereo"[..]]
    );

    // Instantiate the desired plugin
    let mut plugin = PluginInstance::<TestHostHandlers>::new(
        |_| TestHostShared,
        |_| TestHostMainThread,
        &bundle,
        descriptor.id().unwrap(),
        &info,
    )
    .unwrap();

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

    // Setting up some buffers
    let configuration = PluginAudioConfiguration {
        sample_rate: 44_100.0,
        min_frames_count: 32,
        max_frames_count: 32,
    };

    let processor = plugin
        .activate(|_, _| TestHostAudioProcessor, configuration)
        .unwrap();

    assert!(plugin.is_active());

    let mut input_events = EventBuffer::with_capacity(10);
    let mut output_events = EventBuffer::with_capacity(10);

    input_events.push(&ParamValueEvent::new(
        0,
        ClapId::new(1),
        Pckn::match_all(),
        0.5,
        Cookie::empty(),
    ));

    let mut input_buffers = [vec![69f32; 32], vec![69f32; 32]];
    let mut output_buffers = [vec![0f32; 32], vec![0f32; 32]];

    let mut processor = processor.start_processing().unwrap();

    let mut inputs_descriptors = AudioPorts::with_capacity(2, 1);
    let mut outputs_descriptors = AudioPorts::with_capacity(2, 1);

    let input_channels = inputs_descriptors.with_input_buffers([AudioPortBuffer {
        channels: AudioPortBufferType::f32_input_only(
            input_buffers.iter_mut().map(InputChannel::variable),
        ),
        latency: 0,
    }]);

    let mut output_channels = outputs_descriptors.with_output_buffers([AudioPortBuffer {
        channels: AudioPortBufferType::f32_output_only(
            output_buffers.iter_mut().map(|b| b.as_mut_slice()),
        ),
        latency: 0,
    }]);

    processor
        .process(
            &input_channels,
            &mut output_channels,
            &input_events.as_input(),
            &mut output_events.as_output(),
            None,
            None,
        )
        .unwrap();

    // Check the gain was applied properly
    for channel_index in 0..1 {
        let inbuf = &input_buffers[channel_index];
        let outbuf = &output_buffers[channel_index];
        for (input, output) in inbuf.iter().zip(outbuf.iter()) {
            assert_eq!(*output, *input * 0.5)
        }
    }

    plugin.deactivate(processor.stop_processing());
}

#[test]
fn preset_listing_works() {
    // Initialize host with basic info
    let info = HostInfo::new("test", "", "", "").unwrap();

    let bundle = PluginBundle::load_from_clack::<GainPluginEntry>(c"").unwrap();
    let factory = bundle.get_factory::<PresetDiscoveryFactory>().unwrap();
    let providers: Vec<_> = factory.provider_descriptors().collect();
    assert_eq!(providers.len(), 1);
    let provider = providers[0];
    assert_eq!(
        provider.id().unwrap(),
        c"org.rust-audio.clack.gain-presets-provider"
    );

    let provider_id = provider.id().unwrap();
    let mut provider =
        Provider::instantiate(TestIndexer { declared: false }, &bundle, provider_id, &info)
            .unwrap();

    assert!(provider.indexer().declared);

    let mut receiver = TestReceiver {
        presets: Vec::new(),
    };

    provider.get_metadata(Location::Plugin, &mut receiver);

    assert_eq!(
        &receiver.presets,
        &[
            Preset {
                name: c"Unity".to_owned(),
                load_key: c"0".to_owned(),
                plugin_ids: vec![c"org.rust-audio.clack.gain-presets".to_owned()],
            },
            Preset {
                name: c"Quieter".to_owned(),
                load_key: c"1".to_owned(),
                plugin_ids: vec![c"org.rust-audio.clack.gain-presets".to_owned()],
            },
        ]
    );
}

struct TestHostMainThread;
struct TestHostShared;
struct TestHostAudioProcessor;
struct TestHostHandlers;

impl SharedHandler<'_> for TestHostShared {
    fn request_restart(&self) {
        unimplemented!()
    }

    fn request_process(&self) {
        unimplemented!()
    }

    fn request_callback(&self) {
        unimplemented!()
    }
}

impl AudioProcessorHandler<'_> for TestHostAudioProcessor {}

impl MainThreadHandler<'_> for TestHostMainThread {}

impl HostHandlers for TestHostHandlers {
    type Shared<'a> = TestHostShared;
    type MainThread<'a> = TestHostMainThread;
    type AudioProcessor<'a> = TestHostAudioProcessor;
}

struct TestIndexer {
    declared: bool,
}

impl IndexerImpl for TestIndexer {
    fn declare_filetype(&mut self, _file_type: FileType) -> Result<(), HostError> {
        unreachable!()
    }

    fn declare_location(&mut self, location: LocationInfo) -> Result<(), HostError> {
        assert!(!self.declared);
        assert_eq!(location.location, Location::Plugin);
        self.declared = true;

        Ok(())
    }

    fn declare_soundpack(&mut self, _soundpack: Soundpack) -> Result<(), HostError> {
        unreachable!()
    }
}

struct TestReceiver {
    presets: Vec<Preset>,
}

#[derive(PartialEq, Eq, Debug)]
struct Preset {
    name: CString,
    load_key: CString,
    plugin_ids: Vec<CString>,
}

impl MetadataReceiverImpl for TestReceiver {
    fn on_error(&mut self, _error_code: i32, _error_message: Option<&CStr>) {
        unreachable!()
    }

    fn begin_preset(
        &mut self,
        name: Option<&CStr>,
        load_key: Option<&CStr>,
    ) -> Result<(), HostError> {
        self.presets.push(Preset {
            name: name.unwrap().to_owned(),
            load_key: load_key.unwrap().to_owned(),
            plugin_ids: Vec::new(),
        });

        Ok(())
    }

    fn add_plugin_id(&mut self, plugin_id: UniversalPluginId) {
        assert_eq!(
            plugin_id,
            UniversalPluginId::clap(c"org.rust-audio.clack.gain-presets")
        );

        self.presets
            .last_mut()
            .unwrap()
            .plugin_ids
            .push(plugin_id.id.to_owned());
    }

    fn set_soundpack_id(&mut self, _soundpack_id: &CStr) {
        unreachable!()
    }

    fn set_flags(&mut self, _flags: Flags) {
        unreachable!()
    }

    fn add_creator(&mut self, creator: &CStr) {
        assert_eq!(creator, c"Me!")
    }

    fn set_description(&mut self, _description: &CStr) {
        unreachable!()
    }

    fn set_timestamps(
        &mut self,
        _creation_time: Option<Timestamp>,
        _modification_time: Option<Timestamp>,
    ) {
        unreachable!()
    }

    fn add_feature(&mut self, _feature: &CStr) {
        unreachable!()
    }

    fn add_extra_info(&mut self, _key: &CStr, _value: &CStr) {
        unreachable!()
    }
}
