use crate::preset_discovery::indexer::PresetIndexer;
use clack_extensions::preset_discovery::{
    Location, PresetDiscoveryFactory, Provider, ProviderDescriptor,
};
use clack_host::prelude::{HostInfo, PluginBundle};
pub struct PresetDiscoveryData {}

mod indexer;
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
    let mut provider =
        Provider::instantiate(PresetIndexer::new, bundle, descriptor.id()?, host_info).unwrap();

    let location = Location::File {
        path: c"/usr/share/surge-xt/patches_factory/Basses/Smoothie.fxp",
    };

    let metadata = metadata::get_metadata(&mut provider, location).unwrap();
    dbg!(&metadata);

    Some(PresetDiscoveryData {})
}
