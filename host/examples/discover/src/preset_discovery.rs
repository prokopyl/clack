use clack_extensions::preset_discovery::indexer::Indexer;
use clack_extensions::preset_discovery::{
    FileType, Flags, Location, LocationData, MetadataReceiver, PresetDiscoveryFactory, Provider,
    ProviderDescriptor, Soundpack,
};
use clack_host::prelude::{HostInfo, PluginBundle};
use clack_host::utils::{Timestamp, UniversalPluginID};
use std::ffi::CStr;

pub struct PresetDiscoveryData {}

pub fn get_presets(bundle: &PluginBundle) {
    let host_info = HostInfo::new("", "", "", "").unwrap();

    let provider_descriptors =
        if let Some(discovery) = bundle.get_factory::<PresetDiscoveryFactory>() {
            discovery
                .provider_descriptors()
                .filter_map(|d| scan_provider(bundle, d, host_info.clone()))
                .collect()
        } else {
            vec![]
        };
}

pub fn scan_provider(
    bundle: &PluginBundle,
    descriptor: &ProviderDescriptor,
    host_info: HostInfo,
) -> Option<PresetDiscoveryData> {
    let mut provider = Provider::instantiate(
        PresetIndexer::new,
        bundle,
        descriptor.id().unwrap(),
        host_info,
    )
    .unwrap();

    let location = Location::File {
        path: c"/usr/share/surge-xt/patches_factory/Basses/Smoothie.fxp",
    };
    let mut receiver = MyMetadataReceiver {};
    provider.get_metadata(location, &mut receiver);

    Some(PresetDiscoveryData {})
}

struct PresetIndexer {}

impl PresetIndexer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Indexer for PresetIndexer {
    fn declare_filetype(&mut self, file_type: FileType) {
        eprintln!("declared filetype {:?}", file_type);
    }

    fn declare_location(&mut self, location: LocationData) {
        println!("{:?}", location);
    }

    fn declare_soundpack(&mut self, soundpack: Soundpack) {
        println!("{:?}", soundpack);
    }
}

struct MyMetadataReceiver {}

impl MetadataReceiver for MyMetadataReceiver {
    fn on_error(&mut self, error_code: i32, error_message: Option<&CStr>) {
        eprintln!("error code {}, {:?}", error_code, error_message);
    }

    fn begin_preset(&mut self, name: Option<&CStr>, load_key: Option<&CStr>) {
        dbg!(name, load_key);
    }

    fn add_plugin_id(&mut self, plugin_id: UniversalPluginID) {
        dbg!(plugin_id);
    }

    fn set_soundpack_id(&mut self, soundpack_id: &CStr) {
        dbg!(soundpack_id);
    }

    fn set_flags(&mut self, flags: Flags) {
        dbg!(flags);
    }

    fn add_creator(&mut self, creator: &CStr) {
        dbg!(creator);
    }

    fn set_description(&mut self, description: &CStr) {
        dbg!(description);
    }

    fn set_timestamps(
        &mut self,
        creation_time: Option<Timestamp>,
        modification_time: Option<Timestamp>,
    ) {
        dbg!(creation_time, modification_time);
    }

    fn add_feature(&mut self, feature: &CStr) {
        dbg!(feature);
    }

    fn add_extra_info(&mut self, key: &CStr, value: &CStr) {
        dbg!(key, value);
    }
}
