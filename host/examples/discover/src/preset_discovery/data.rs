use crate::preset_discovery::indexer::Location;
use crate::preset_discovery::metadata::PresetData;
use std::path::Path;

#[derive(Debug)]
pub enum PresetsAtLocation {
    Plugin {
        location: Location,
        presets: Vec<PresetData>,
    },
    Files {
        location: Location,
        files: Vec<PresetsInFile>,
    },
}

#[derive(Debug)]
pub struct PresetsInFile {
    pub path: Box<Path>,
    pub presets: Vec<PresetData>,
}

#[derive(Debug)]
pub struct PresetDiscoveryData {
    pub presets_per_location: Vec<PresetsAtLocation>,
}
