use clack_extensions::preset_discovery::indexer::Indexer;
use clack_extensions::preset_discovery::{
    FileType, LocationData, Provider, ProviderDescriptor, Soundpack,
};
use clack_host::prelude::{HostInfo, PluginBundle};
use std::ffi::CStr;

pub struct PresetDiscoveryData {}

pub fn get_presets(
    bundle: &PluginBundle,
    descriptor: &ProviderDescriptor,
    host_info: HostInfo,
) -> Option<PresetDiscoveryData> {
    let provider = Provider::instantiate(
        PresetIndexer::new,
        bundle,
        descriptor.id().unwrap(),
        host_info,
    )
    .unwrap();

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
