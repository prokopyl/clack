use clack_extensions::preset_discovery::{self, prelude::*};
use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
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

impl IndexerImpl for PresetIndexer {
    fn declare_filetype(&mut self, file_type: preset_discovery::preset_data::FileType) {
        self.file_types.push(FileType {
            name: file_type.name.to_owned().into_boxed_c_str(),
            description: file_type
                .description
                .map(|c| c.to_owned().into_boxed_c_str()),
            extension: file_type.description.map(path_from_c_str),
        })
    }

    fn declare_location(&mut self, location: preset_discovery::preset_data::LocationInfo) {
        self.locations.push(Location {
            flags: location.flags,
            name: location.name.to_owned().into_boxed_c_str(),
            file_path: location.location.file_path().map(path_from_c_str),
        });
    }

    fn declare_soundpack(&mut self, soundpack: preset_discovery::preset_data::Soundpack) {
        self.soundpacks.push(Soundpack {
            flags: soundpack.flags,
            id: soundpack.id.to_owned().into_boxed_c_str(),
            name: soundpack.name.to_owned().into_boxed_c_str(),
        })
    }
}

#[derive(Debug)]
pub struct FileType {
    pub name: Box<CStr>,
    pub description: Option<Box<CStr>>,
    pub extension: Option<Box<Path>>,
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.to_string_lossy())?;

        match &self.extension {
            Some(extension) => write!(f, "<*.{}>", extension.to_string_lossy())?,
            None => write!(f, " <all file extensions>")?,
        };

        if let Some(description) = &self.description {
            if !description.is_empty() {
                write!(f, ": {}", description.to_string_lossy())?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Location {
    pub flags: Flags,
    pub name: Box<CStr>,
    pub file_path: Option<Box<Path>>,
}

#[derive(Debug)]
pub struct Soundpack {
    pub flags: Flags,
    pub id: Box<CStr>,
    pub name: Box<CStr>,
}

impl Display for Soundpack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.to_string_lossy())?;
        write!(f, " <{}>", self.id.to_string_lossy())?;
        write!(f, " ({:?})", self.flags)?;

        Ok(())
    }
}

fn path_from_c_str(cstr: &CStr) -> Box<Path> {
    match cstr.to_string_lossy() {
        Cow::Borrowed(string) => PathBuf::from(string),
        Cow::Owned(string) => PathBuf::from(string),
    }
    .into()
}
