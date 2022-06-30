use clack_host::bundle::PluginBundle;
use clack_host::factory::PluginFactory;
use clack_host::host::{AudioProcessorHoster, HostInfo, PluginHost, PluginHoster, SharedHoster};
use clack_host::instance::{PluginAudioConfiguration, PluginInstance};

#[test]
#[cfg_attr(miri, ignore)] // Miri does not support calling foreign function (dlopen)
pub fn it_works() {
    let bundle_path = format!(
        "{}/../target/debug/{}gain{}",
        env!("CARGO_MANIFEST_DIR"),
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let bundle = PluginBundle::load(&bundle_path).unwrap();

    let desc = bundle
        .get_factory::<PluginFactory>()
        .unwrap()
        .plugin_descriptor(0)
        .unwrap();
    assert_eq!(desc.id().unwrap().to_bytes(), b"gain");
}

#[test]
#[cfg_attr(miri, ignore)] // Miri does not support calling foreign function (dlopen)
pub fn it_works_2() {
    struct PH;
    struct SH;
    struct AH;

    impl<'a> AudioProcessorHoster for AH {}
    impl<'a> SharedHoster<'a> for SH {
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

    impl<'a> PluginHoster<'a> for PH {
        type AudioProcessor = AH;
        type Shared = SH;
    }

    let plugin =
        Box::new(PluginBundle::load("/home/adrien/.clap/clack_example_gain_debug.clap").unwrap());
    let plugin = Box::leak(plugin);
    let host = PluginHost::new(
        HostInfo::new("clapjack", "jaxter184", "net.jaxter184.clapjack", "0.0.1").unwrap(),
    );
    /*let mut instance = PluginInstance::new(|_| SH, |_| PH, &entry, &[], &host).unwrap();
    let config = PluginAudioConfiguration {
        sample_rate: 44100.0,
        frames_count_range: 16..=2048,
    };
    let stopped = instance.activate(|_, _| AH, config).unwrap();
    let _processor = stopped.start_processing().unwrap();*/
}
