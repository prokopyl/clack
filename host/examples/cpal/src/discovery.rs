// Discovering plugins means loading them, which is unsafe
#![allow(unsafe_code)]

use clack_finder::{ClapFinder, PotentialClapFile};
use clack_host::entry::{LibraryEntry, PluginEntryError};
use clack_host::prelude::*;
use rayon::prelude::*;
use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

/// The descriptor of a plugin that was found, alongside the bundle it was loaded from, as well
/// as its path.
pub struct FoundPlugin {
    /// The plugin's descriptor.
    pub plugin: PluginDescriptor,
    /// The bundle the descriptor was loaded from.
    pub entry: PluginEntry,
    /// The path of the entry's bundle file.
    pub path: PathBuf,
}

/// Scans the CLAP standard paths for all the plugin descriptors that match the given ID.
pub fn scan_for_plugin_id(id: &str) -> Vec<FoundPlugin> {
    let standard_paths = clack_finder::standard_clap_paths();

    println!("Scanning the following directories for CLAP plugin with ID {id}:");
    for path in &standard_paths {
        println!("\t - {}", path.display())
    }

    let found_files: Vec<PotentialClapFile> = ClapFinder::new(standard_paths).into_iter().collect();
    println!("\t * Found {} potential CLAP entries.", found_files.len());

    scan_plugins(&found_files, id)
}

/// Loads all the given potential CLAP files, and returns a list of all the plugins that match the given ID.
fn scan_plugins(files: &[PotentialClapFile], searched_id: &str) -> Vec<FoundPlugin> {
    files
        .par_iter()
        .filter_map(|p| scan_file(p, searched_id))
        .collect()
}

/// Scans a given file, looking for a plugin matching the given ID.
/// If this file wasn't a CLAP or doesn't contain a plugin with a given ID, this returns `None`.
fn scan_file(file: &PotentialClapFile, searched_id: &str) -> Option<FoundPlugin> {
    let bundle_path = CString::new(file.bundle_path().to_string_lossy().into_owned()).ok()?;

    let library = unsafe { LibraryEntry::load_from_path(file.executable_path()) }.ok()?;
    let entry = unsafe { PluginEntry::load_from(library, &bundle_path) }.ok()?;

    for plugin in entry.get_plugin_factory()?.plugin_descriptors() {
        let Some(plugin) = PluginDescriptor::try_from(plugin) else {
            continue;
        };

        if plugin.id == searched_id {
            return Some(FoundPlugin {
                plugin,
                entry,
                path: file.bundle_path().to_path_buf(),
            });
        }
    }

    None
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

/// Lists all plugins in a given file.
pub fn list_plugins_in_file(file: &Path) -> Result<Vec<FoundPlugin>, DiscoveryError> {
    let entry = unsafe { PluginEntry::load(file)? };
    let Some(plugin_factory) = entry.get_plugin_factory() else {
        return Err(DiscoveryError::MissingPluginFactory);
    };

    Ok(plugin_factory
        .plugin_descriptors()
        .filter_map(PluginDescriptor::try_from)
        .map(|plugin| FoundPlugin {
            entry: entry.clone(),
            path: file.to_path_buf(),
            plugin,
        })
        .collect())
}

/// Errors that happened during finder.
#[derive(Debug)]
pub enum DiscoveryError {
    /// Unable to load a CLAP file.
    LoadError(PluginEntryError),
    /// The CLAP file has no plugin factory.
    MissingPluginFactory,
}

impl From<PluginEntryError> for DiscoveryError {
    fn from(value: PluginEntryError) -> Self {
        Self::LoadError(value)
    }
}

impl Display for DiscoveryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::LoadError(e) => write!(f, "Failed to load plugin file: {e}"),
            DiscoveryError::MissingPluginFactory => f.write_str("File has no plugin factory"),
        }
    }
}

impl Error for DiscoveryError {}

/// Loads a specific ID from a specific file's path.
pub fn load_plugin_id_from_path(
    file_path: &Path,
    id: &str,
) -> Result<Option<FoundPlugin>, DiscoveryError> {
    let entry = unsafe { PluginEntry::load(file_path)? };
    let Some(plugin_factory) = entry.get_plugin_factory() else {
        return Err(DiscoveryError::MissingPluginFactory);
    };

    Ok(plugin_factory
        .plugin_descriptors()
        .filter_map(PluginDescriptor::try_from)
        .find(|p| p.id == id)
        .map(|plugin| FoundPlugin {
            plugin,
            entry,
            path: file_path.to_path_buf(),
        }))
}
