use clack_host::bundle::PluginBundleError;
use clack_host::prelude::*;
use rayon::prelude::*;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub struct FoundBundlePlugin {
    pub bundle: PluginBundle,
    pub path: PathBuf,
    pub plugin: PluginDescriptor,
}

pub fn scan_for_plugin_id(id: &str) -> Vec<FoundBundlePlugin> {
    let standard_paths = standard_clap_paths();

    println!("Scanning the following directories for CLAP plugin with ID {id}:");
    for path in &standard_paths {
        println!("\t - {}", path.display())
    }

    let found_bundles = walk_paths(&standard_paths);
    println!("\t * Found {} potential CLAP bundles.", found_bundles.len());

    scan_plugins(&found_bundles, id)
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
        paths.push("/usr/lib/clap".into())
    }

    #[cfg(target_family = "unix")]
    fn split_path(path: &OsStr) -> impl Iterator<Item = OsString> + '_ {
        use std::os::unix::ffi::OsStrExt;
        path.as_bytes()
            .split(|c| *c == b':')
            .map(|bytes| OsStr::from_bytes(bytes).to_os_string())
    }

    #[cfg(target_os = "windows")]
    fn split_path(path: &OsStr) -> impl Iterator<Item = OsString> + '_ {
        use std::os::windows::ffi::*;
        let buf: Vec<u16> = path.encode_wide().collect();
        buf.split(|c| *c == b';'.into()).map(OsString::from_wide)
    }

    if let Some(env_var) = std::env::var_os("CLAP_PATH") {
        paths.extend(split_path(&env_var).map(PathBuf::from))
    }

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

fn scan_plugins(bundles: &[DirEntry], searched_id: &str) -> Vec<FoundBundlePlugin> {
    bundles
        .par_iter()
        .filter_map(|p| scan_plugin(p.path(), searched_id))
        .collect()
}

fn scan_plugin(path: &Path, searched_id: &str) -> Option<FoundBundlePlugin> {
    let Ok(bundle) = PluginBundle::load(path) else { return None; };
    for plugin in bundle.get_plugin_factory()?.plugin_descriptors() {
        let Some(plugin) = PluginDescriptor::try_from(plugin) else { continue };

        if plugin.id == searched_id {
            return Some(FoundBundlePlugin {
                plugin,
                bundle,
                path: path.to_path_buf(),
            });
        }
    }

    None
}

#[derive(Debug)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<String>,
}

impl PluginDescriptor {
    pub fn try_from(p: clack_host::factory::PluginDescriptor) -> Option<Self> {
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

/// Lists all plugins in a given bundle.
pub fn list_plugins_in_bundle(
    bundle_path: &Path,
) -> Result<Vec<FoundBundlePlugin>, DiscoveryError> {
    let bundle = PluginBundle::load(bundle_path)?;
    let Some(plugin_factory) = bundle.get_plugin_factory() else { return Err(DiscoveryError::MissingPluginFactory) };

    Ok(plugin_factory
        .plugin_descriptors()
        .filter_map(PluginDescriptor::try_from)
        .map(|plugin| FoundBundlePlugin {
            bundle: bundle.clone(),
            path: bundle_path.to_path_buf(),
            plugin,
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

pub fn load_plugin_id_from_path(
    bundle_path: &Path,
    id: &str,
) -> Result<Option<FoundBundlePlugin>, DiscoveryError> {
    let bundle = PluginBundle::load(bundle_path)?;
    let Some(plugin_factory) = bundle.get_plugin_factory() else { return Err(DiscoveryError::MissingPluginFactory) };

    Ok(plugin_factory
        .plugin_descriptors()
        .filter_map(PluginDescriptor::try_from)
        .find(|p| p.id == id)
        .map(|plugin| FoundBundlePlugin {
            plugin,
            bundle,
            path: bundle_path.to_path_buf(),
        }))
}
