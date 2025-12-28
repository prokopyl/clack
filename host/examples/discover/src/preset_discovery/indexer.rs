use clack_extensions::preset_discovery::indexer::Indexer;
use clack_extensions::preset_discovery::{self, Flags};
use std::borrow::Cow;
use std::ffi::CStr;
use std::path::{Path, PathBuf};

pub struct PresetIndexer {
    pub file_types: Vec<FileType>,
    pub locations: Vec<Location>,
    pub soundpacks: Vec<Soundpack>,
}

impl PresetIndexer {
    pub fn new() -> Self {
        Self {
            file_types: Vec::new(),
            locations: Vec::new(),
            soundpacks: Vec::new(),
        }
    }

    pub fn take(&mut self) -> Self {
        core::mem::replace(self, Self::new())
    }
}

impl Indexer for PresetIndexer {
    fn declare_filetype(&mut self, file_type: preset_discovery::FileType) {
        self.file_types.push(FileType {
            name: file_type.name.to_owned().into_boxed_c_str(),
            description: file_type
                .description
                .map(|c| c.to_owned().into_boxed_c_str()),
            extension: file_type.description.map(path_from_c_str),
        })
    }

    fn declare_location(&mut self, location: preset_discovery::LocationData) {
        self.locations.push(Location {
            flags: location.flags,
            name: location.name.to_owned().into_boxed_c_str(),
            file_path: location.location.file_path().map(path_from_c_str),
        });
    }

    fn declare_soundpack(&mut self, soundpack: preset_discovery::Soundpack) {
        self.soundpacks.push(Soundpack {
            flags: soundpack.flags,
            id: soundpack.id.to_owned().into_boxed_c_str(),
            name: soundpack.name.to_owned().into_boxed_c_str(),
        })
    }
}

pub struct FileType {
    pub name: Box<CStr>,
    pub description: Option<Box<CStr>>,
    pub extension: Option<Box<Path>>,
}

pub struct Location {
    pub flags: Flags,
    pub name: Box<CStr>,
    pub file_path: Option<Box<Path>>,
}

pub struct Soundpack {
    pub flags: Flags,
    pub id: Box<CStr>,
    pub name: Box<CStr>,
}

fn path_from_c_str(cstr: &CStr) -> Box<Path> {
    match cstr.to_string_lossy() {
        Cow::Borrowed(string) => PathBuf::from(string),
        Cow::Owned(string) => PathBuf::from(string),
    }
    .into()
}
