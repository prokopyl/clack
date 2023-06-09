use clack_host::bundle::PluginBundleError;
use clack_host::prelude::*;
use directories::BaseDirs;
use rayon::prelude::*;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub struct FoundBundle {
    bundle: PluginBundle,
    path: PathBuf,
}

pub fn scan_for_plugin_id(id: &str) -> Vec<FoundBundle> {
    let standard_paths = standard_clap_paths();

    println!("Scanning the following directories for CLAP plugins:");
    for path in &standard_paths {
        println!("\t{}", path.display())
    }

    let found_bundles = walk_paths(&standard_paths);
    println!("Found {} potential CLAP bundles:", found_bundles.len());
    for x in &found_bundles {
        println!("Found {}", x.path().display())
    }

    println!("Scanning bundles for plugins…");
    scan_plugins(&found_bundles);
    // Step 2: Walkdir over each to find `.clap` files.
    // Step 3: for each one of them, load the dylib and filter on available plugins
    // Step 4: collect them all into vec
    // todo!()
    vec![]
}

//
// Linux
//   - ~/.clap
//   - /usr/lib/clap
//
// Windows
//   - %COMMONPROGRAMFILES%\CLAP
//   - %LOCALAPPDATA%\Programs\Common\CLAP
//
// MacOS
//   - /Library/Audio/Plug-Ins/CLAP
//   - ~/Library/Audio/Plug-Ins/CLAP
//
// In addition to the OS-specific default locations above, a CLAP host must query the environment
// for a CLAP_PATH variable, which is a list of directories formatted in the same manner as the host
// OS binary search path (PATH on Unix, separated by `:` and Path on Windows, separated by ';', as
// of this writing).
//
fn standard_clap_paths() -> Vec<PathBuf> {
    let mut paths = vec![];

    if let Some(dirs) = BaseDirs::new() {
        paths.push(dirs.home_dir().join(".clap"))
    }

    #[cfg(target_family = "unix")]
    {
        paths.push("/usr/lib/clap".into())
    }

    // TODO: windows, macOS, ENV

    paths
}

fn is_clap_bundle(dir_entry: &DirEntry) -> bool {
    dir_entry.file_type().is_file() && dir_entry.file_name().to_string_lossy().ends_with(".clap")
}

fn walk_paths(search_dirs: &[PathBuf]) -> Vec<DirEntry> {
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

fn scan_plugins(bundles: &[DirEntry]) {
    bundles.par_iter().for_each(|p| scan_plugin(p.path()));
}

fn scan_plugin(path: &Path) {
    let Ok(bundle) = PluginBundle::load(path) else { return; };
    for plugin in bundle.get_plugin_factory().unwrap().plugin_descriptors() {
        let Some(id) = plugin.id() else { continue };
        println!("Found {} at {}", id.to_string_lossy(), path.display());
    }
}

#[derive(Debug)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<String>,
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

/// Lists all plugins in a given bundle.
pub fn list_plugins_in_bundle(bundle_path: &Path) -> Result<Vec<PluginDescriptor>, DiscoveryError> {
    let bundle = PluginBundle::load(bundle_path)?;
    let Some(plugin_factory) = bundle.get_plugin_factory() else { return Err(DiscoveryError::MissingPluginFactory) };

    Ok(plugin_factory
        .plugin_descriptors()
        .filter_map(|p| {
            Some(PluginDescriptor {
                id: p.id()?.to_str().ok()?.to_string(),
                version: p
                    .version()
                    .filter(|s| !s.to_bytes().is_empty())
                    .map(|v| v.to_string_lossy().to_string()),
                name: p
                    .name()
                    .filter(|s| !s.to_bytes().is_empty())
                    .map(|v| v.to_string_lossy().to_string()),
            })
        })
        .collect())
}

#[derive(Debug)]
pub enum DiscoveryError {
    LoadError(PluginBundleError),
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
