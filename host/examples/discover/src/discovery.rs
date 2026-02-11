use clack_finder::{ClapBundle, ClapFinder};
use clack_host::bundle::LibraryEntry;
use clack_host::prelude::*;
use rayon::prelude::*;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

/// The descriptor of a plugin that was found, alongside the bundle it was loaded from, as well
/// as its path.
pub struct FoundBundlePlugin {
    /// The plugin's descriptor.
    pub plugins: Vec<PluginDescriptor>,
    /// The bundle the descriptor was loaded from.
    pub bundle: PluginBundle,
    /// The path of the bundle's file.
    pub path: PathBuf,
}

/// Search the given directories' contents, and returns a list of all the files that could be CLAP
/// bundles.
pub fn search_for_potential_bundles(search_dirs: Vec<PathBuf>) -> Vec<ClapBundle> {
    ClapFinder::new(search_dirs).into_iter().collect()
}

/// Loads all the given bundles, and returns a list of all the plugins in them.
pub fn scan_bundles(bundles: Vec<ClapBundle>) -> Vec<FoundBundlePlugin> {
    bundles
        .into_par_iter()
        .filter_map(|p| scan_plugin(p))
        .collect()
}

/// Loads all the given bundles, and returns a list of all the plugins that match the given ID.
pub fn scan_bundles_matching(bundles: Vec<ClapBundle>, plugin_id: &str) -> Vec<FoundBundlePlugin> {
    bundles
        .into_par_iter()
        .filter_map(|p| scan_plugin(p))
        .filter(|p| p.plugins.iter().any(|p| p.id == plugin_id))
        .collect()
}

/// Scans a given bundle, looking for a plugin matching the given ID.
/// If this file wasn't a bundle or doesn't contain a plugin with a given ID, this returns `None`.
pub fn scan_plugin(path: ClapBundle) -> Option<FoundBundlePlugin> {
    let bundle_path = CString::new(path.bundle_path().to_string_lossy().into_owned()).ok()?;

    let library = unsafe { LibraryEntry::load_from_path(path.executable_path()) }.ok()?;
    let bundle = unsafe { PluginBundle::load_from(library, &bundle_path) }.ok()?;

    scan_bundle(bundle, path.into_bundle_path())
}

pub fn scan_plugin_from_path(path: PathBuf) -> Option<FoundBundlePlugin> {
    let bundle = unsafe { PluginBundle::load(&path) }.ok()?;

    scan_bundle(bundle, path)
}

fn scan_bundle(bundle: PluginBundle, bundle_path: PathBuf) -> Option<FoundBundlePlugin> {
    Some(FoundBundlePlugin {
        plugins: bundle
            .get_plugin_factory()?
            .plugin_descriptors()
            .filter_map(PluginDescriptor::try_from)
            .collect(),
        bundle,
        path: bundle_path,
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
