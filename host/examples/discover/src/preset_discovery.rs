use clack_extensions::preset_discovery::indexer::Indexer;
use clack_extensions::preset_discovery::{
    FileType, Flags, Location, LocationData, MetadataReceiver, PresetDiscoveryFactory, Provider,
    ProviderDescriptor, Soundpack,
};
use clack_host::prelude::{HostInfo, PluginBundle};
use clack_host::utils::{Timestamp, UniversalPluginID};
use std::ffi::{CStr, CString};

pub struct PresetDiscoveryData {}

mod metadata;

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

    let metadata = metadata::get_metadata(&mut provider, location).unwrap();
    dbg!(&metadata);

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
