use crate::preset_discovery::data::{PresetDiscoveryData, PresetsAtLocation, PresetsInFile};
use crate::preset_discovery::indexer::{FileType, Location, PresetIndexer};
use clack_extensions::preset_discovery::{
    self, PresetDiscoveryFactory, Provider, ProviderDescriptor,
};
use clack_host::prelude::{HostInfo, PluginBundle};
use std::ffi::CString;
use walkdir::{DirEntry, WalkDir};

pub mod data;
mod indexer;
mod metadata;

pub fn get_presets(bundle: &PluginBundle) -> Vec<PresetDiscoveryData> {
    let host_info = HostInfo::new("", "", "", "").unwrap();

    if let Some(discovery) = bundle.get_factory::<PresetDiscoveryFactory>() {
        discovery
            .provider_descriptors()
            .filter_map(|d| scan_provider(bundle, d, host_info.clone()))
            .collect()
    } else {
        vec![]
    }
}

pub fn scan_provider(
    bundle: &PluginBundle,
    descriptor: &ProviderDescriptor,
    host_info: HostInfo,
) -> Option<PresetDiscoveryData> {
    let mut provider =
        Provider::instantiate(PresetIndexer::new, bundle, descriptor.id()?, host_info).unwrap();

    let indexer_data = provider.indexer_mut().take();

    let presets_per_location = indexer_data
        .locations
        .into_iter()
        .map(|l| scan_location(&mut provider, l, &indexer_data.file_types))
        .collect();

    Some(PresetDiscoveryData {
        presets_per_location,
    })
}

fn scan_location(
    provider: &mut Provider<PresetIndexer>,
    location: Location,
    file_types: &[FileType],
) -> PresetsAtLocation {
    let Some(file_path) = &location.file_path else {
        return PresetsAtLocation::Plugin {
            location,
            presets: metadata::get_metadata(provider, preset_discovery::Location::Plugin).unwrap(),
        };
    };

    let files = WalkDir::new(file_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| file_matches(e, file_types))
        .filter_map(|e| {
            let path_c_str = CString::new(e.path().to_str()?).ok()?;
            Some(PresetsInFile {
                presets: metadata::get_metadata(
                    provider,
                    preset_discovery::Location::File { path: &path_c_str },
                )
                .ok()?,
                path: e.into_path().into_boxed_path(),
            })
        })
        .collect();

    PresetsAtLocation::Files { location, files }
}

fn file_matches(entry: &DirEntry, file_types: &[FileType]) -> bool {
    if file_types.is_empty() {
        return true;
    }

    let extension = entry.path().extension();

    file_types.iter().any(|f| match (&f.extension, &extension) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(ext), Some(file_ext)) => ext.as_os_str() == *file_ext,
    })
}
