use clack_host::bundle::PluginBundleError;
use clack_host::prelude::*;
use rayon::prelude::*;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

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

/// Returns a list of all the standard CLAP search paths, per the CLAP specification.
pub fn standard_clap_paths() -> Vec<PathBuf> {
    let mut paths = vec![];

    if let Some(home_dir) = dirs::home_dir() {
        paths.push(home_dir.join(".clap"));

        #[cfg(target_os = "macos")]
        {
            paths.push(home_dir.join("Library/Audio/Plug-Ins/CLAP"));
        }
    }

    #[cfg(windows)]
    {
        if let Some(val) = std::env::var_os("CommonProgramFiles") {
            paths.push(PathBuf::from(val).join("CLAP"))
        }

        if let Some(dir) = dirs::config_local_dir() {
            paths.push(dir.join("Programs\\Common\\CLAP"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Audio/Plug-Ins/CLAP"));
    }

    #[cfg(target_family = "unix")]
    {
        paths.push("/usr/lib/clap".into());
        paths.push("/usr/lib64/clap".into());
    }

    if let Some(env_var) = std::env::var_os("CLAP_PATH") {
        paths.extend(std::env::split_paths(&env_var))
    }

    paths
}

/// Returns `true` if the given entry could refer to a CLAP bundle.
///
/// CLAP bundles are files that end with the `.clap` extension.
fn is_clap_bundle(dir_entry: &DirEntry) -> bool {
    dir_entry.file_type().is_file() && dir_entry.file_name().to_string_lossy().ends_with(".clap")
}

/// Search the given directories' contents, and returns a list of all the files that could be CLAP
/// bundles.
pub fn search_for_potential_bundles(search_dirs: &[PathBuf]) -> Vec<DirEntry> {
    search_dirs
        .iter()
        .flat_map(|path| {
            WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(is_clap_bundle)
        })
        .collect()
}

/// Loads all the given bundles, and returns a list of all the plugins in them.
pub fn scan_bundles(bundles: &[DirEntry]) -> Vec<FoundBundlePlugin> {
    bundles
        .par_iter()
        .filter_map(|p| scan_plugin(p.path()))
        .collect()
}

/// Loads all the given bundles, and returns a list of all the plugins that match the given ID.
pub fn scan_bundles_matching(bundles: &[DirEntry], plugin_id: &str) -> Vec<FoundBundlePlugin> {
    bundles
        .par_iter()
        .filter_map(|p| scan_plugin(p.path()))
        .filter(|p| p.plugins.iter().any(|p| p.id == plugin_id))
        .collect()
}

/// Scans a given bundle, looking for a plugin matching the given ID.
/// If this file wasn't a bundle or doesn't contain a plugin with a given ID, this returns `None`.
pub fn scan_plugin(path: &Path) -> Option<FoundBundlePlugin> {
    let Ok(bundle) = (unsafe { PluginBundle::load(path) }) else {
        return None;
    };

    Some(FoundBundlePlugin {
        plugins: bundle
            .get_plugin_factory()?
            .plugin_descriptors()
            .filter_map(PluginDescriptor::try_from)
            .collect(),
        bundle,
        path: path.to_path_buf(),
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

/// Errors that happened during discovery.
#[derive(Debug)]
pub enum DiscoveryError {
    /// Unable to load a CLAP bundle.
    LoadError(PluginBundleError),
    /// The CLAP bundle has no plugin factory.
    MissingPluginFactory,
}

impl From<PluginBundleError> for DiscoveryError {
    fn from(value: PluginBundleError) -> Self {
        Self::LoadError(value)
    }
}

impl Display for DiscoveryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::LoadError(e) => write!(f, "Failed to load plugin bundle: {e}"),
            DiscoveryError::MissingPluginFactory => f.write_str("Bundle has no plugin factory"),
        }
    }
}

impl Error for DiscoveryError {}
