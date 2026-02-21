use clack_finder::{ClapFinder, PotentialClapFile};
use clack_host::entry::LibraryEntry;
use clack_host::prelude::*;
use rayon::prelude::*;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// A plugin entry that was successfully scanned for the plugins it exposes.
pub struct ScannedClapFile {
    /// The plugins' descriptors.
    pub plugins: Vec<PluginDescriptor>,
    /// The entry the descriptors were loaded from.
    pub entry: PluginEntry,
    /// The path of the entry's bundle file.
    pub path: PathBuf,
}

/// Search the given directories' contents, and returns a list of all the files that could be CLAP
/// files.
pub fn search_for_potential_files(search_dirs: Vec<PathBuf>) -> Vec<PotentialClapFile> {
    ClapFinder::new(search_dirs).into_iter().collect()
}

/// Loads all the given files, and returns a list of all the plugins in them.
pub fn scan_files(files: Vec<PotentialClapFile>) -> Vec<ScannedClapFile> {
    files.into_par_iter().filter_map(scan_file).collect()
}

/// Loads all the given files, and returns a list of all the plugins that match the given ID.
pub fn scan_files_matching(files: Vec<PotentialClapFile>, plugin_id: &str) -> Vec<ScannedClapFile> {
    files
        .into_par_iter()
        .filter_map(scan_file)
        .filter(|p| p.plugins.iter().any(|p| p.id == plugin_id))
        .collect()
}

/// Scans a given file, looking for a plugin matching the given ID.
/// If this file wasn't a CLAP file or doesn't contain a plugin with a given ID, this returns `None`.
pub fn scan_file(file: PotentialClapFile) -> Option<ScannedClapFile> {
    let bundle_path = CString::new(file.bundle_path().to_string_lossy().into_owned()).ok()?;

    let library = unsafe { LibraryEntry::load_from_path(file.executable_path()) }.ok()?;
    let entry = unsafe { PluginEntry::load_from(library, &bundle_path) }.ok()?;

    scan_entry(entry, file.into_bundle_path())
}

pub fn scan_plugin_from_path(path: PathBuf) -> Option<ScannedClapFile> {
    let entry = unsafe { PluginEntry::load(&path) }.ok()?;

    scan_entry(entry, path)
}

fn scan_entry(entry: PluginEntry, path: PathBuf) -> Option<ScannedClapFile> {
    Some(ScannedClapFile {
        plugins: entry
            .get_plugin_factory()?
            .plugin_descriptors()
            .filter_map(PluginDescriptor::try_from)
            .collect(),
        entry,
        path,
    })
}

/// Simple description of a plugin.
///
/// This is a simplified and owned version of Clack's `PluginDescriptor`.
#[derive(Debug)]
pub struct PluginDescriptor {
    /// The ID of the plugin.
    pub id: String,
    /// (Optional) the user-friendly name of this plugin.
    pub name: Option<String>,
    /// (Optional) the version of this plugin.
    pub version: Option<String>,
}

impl PluginDescriptor {
    /// Reads the description from a given plugin descriptor.
    ///
    /// If the plugin somehow doesn't have a valid ID, this returns `None`.
    pub fn try_from(p: &clack_host::plugin::PluginDescriptor) -> Option<Self> {
        Some(PluginDescriptor {
            id: p.id()?.to_str().ok()?.to_string(),
            version: p.version().map(|v| v.to_string_lossy().to_string()),
            name: p.name().map(|v| v.to_string_lossy().to_string()),
        })
    }
}

impl Display for PluginDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (&self.name, &self.version) {
            (None, None) => write!(f, "{}", &self.id),
            (Some(name), None) => write!(f, "{} ({})", name, &self.id),
            (None, Some(version)) => write!(f, "{} <version {}>", &self.id, version),
            (Some(name), Some(version)) => {
                write!(f, "{} ({}) <version {}>", name, &self.id, version)
            }
        }
    }
}
