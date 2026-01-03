use crate::preset_discovery::indexer::Location;
use crate::preset_discovery::metadata::PresetData;
use std::fmt::{Display, Formatter};
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

impl Display for PresetsAtLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PresetsAtLocation::Plugin { location, presets } => {
                writeln!(
                    f,
                    "-> {} presets in '{}' (in-plugin, flags: {:?})",
                    presets.len(),
                    location.name.to_string_lossy(),
                    location.flags
                )?;

                for preset in presets {
                    writeln!(f, "\t {preset}")?;
                }
            }
            PresetsAtLocation::Files { location, files } => {
                write!(
                    f,
                    "{} preset files in '{}' (at {:?}, flags: {:?})",
                    files.len(),
                    location.name.to_string_lossy(),
                    location.file_path.as_ref().unwrap(),
                    location.flags
                )?;
            }
        };

        Ok(())
    }
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
