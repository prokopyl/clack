#![doc = include_str!("../README.md")]
#![deny(missing_docs, clippy::missing_docs_in_private_items, unsafe_code)]

/// Procedures for discovering and loading CLAP plugin bundles.
mod discovery;
/// The host implementation in itself, for actually running a plugin.
mod host;

use clap::Parser;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::exit;

/// A simple CLI host to load and run a single CLAP plugin.
///
/// At least one of the `--plugin-id` (`-p`) or the `--bundle-path` (`-b`) parameters *must* be used
/// to specify which plugin to load.
#[derive(Parser)]
#[command(about, long_about)]
struct Cli {
    /// Loads the plugin found in the CLAP bundle at the given path.
    ///
    /// If the bundle contains multiple plugins, this should be used in conjunction with the
    /// `--plugin-id` (`-p`) parameter to specify which one to load.
    #[arg(short = 'b', long = "bundle-path")]
    bundle_path: Option<PathBuf>,
    /// Loads the CLAP plugin with the given unique ID.
    ///
    /// This will start to scan the filesystem in the standard CLAP paths, and load all CLAP bundles
    /// found in those paths to search for the plugin matching the given ID.
    ///
    /// If multiple plugins matching the given ID were found on the filesystem, this should be used
    /// in conjunction with the `--bundle-path` (`-b`) parameter to specify which file to load the
    /// plugin from.
    #[arg(short = 'p', long = "plugin-id")]
    plugin_id: Option<String>,
}

fn main() {
    let args = Cli::parse();

    // Select the loading strategy depending on the given arguments
    let result = match (&args.bundle_path, &args.plugin_id) {
        (Some(path), None) => run_from_path(path),
        (None, Some(id)) => run_from_id(id),
        (Some(path), Some(id)) => run_specific(path, id),
        (None, None) => Err(MainError::UnspecifiedOptions.into()),
    };

    if let Err(e) = result {
        eprintln!("{e}");
        exit(1);
    }
}

/// Loads the plugin contained in a bundle, given through its path.
///
/// Returns an error if there is more than one plugin in the bundle.
fn run_from_path(path: &Path) -> Result<(), Box<dyn Error>> {
    let plugins = discovery::list_plugins_in_bundle(path)?;

    if plugins.is_empty() {
        return Err(MainError::NoPluginInPath(path.to_path_buf()).into());
    }

    println!(
        "Found {} plugins in CLAP bundle: {}",
        plugins.len(),
        path.display()
    );

    for p in &plugins {
        println!("\t > {}", &p.plugin)
    }

    if plugins.len() == 1 {
        let plugin = plugins.into_iter().next().unwrap();
        host::run(plugin)
    } else {
        Err(MainError::MultiplePluginsInPath(path.to_path_buf()).into())
    }
}

/// Scans the filesystem to find a plugin with a given ID.
///
/// Returns an error if there is more than one plugin with this ID on the system.
fn run_from_id(id: &str) -> Result<(), Box<dyn Error>> {
    let plugins = discovery::scan_for_plugin_id(id);

    if plugins.is_empty() {
        return Err(MainError::NoPluginWithId(id.to_string()).into());
    }

    println!("Found {} CLAP plugins with id {}:", plugins.len(), id);

    for p in &plugins {
        println!("\t > {} in {}", &p.plugin, p.path.display())
    }

    if plugins.len() == 1 {
        let plugin = plugins.into_iter().next().unwrap();
        host::run(plugin)
    } else {
        Err(MainError::MultiplePluginsWithId(id.to_string()).into())
    }
}

/// Loads a specific plugin matching the given ID, from a specific bundle's path.
///
/// Returns an error if that specific plugin isn't present in the bundle file.
fn run_specific(path: &Path, id: &str) -> Result<(), Box<dyn Error>> {
    let bundle = discovery::load_plugin_id_from_path(path, id)?;

    if let Some(bundle) = bundle {
        host::run(bundle)
    } else {
        Err(MainError::NoPluginInPathWithId(path.to_path_buf(), id.to_string()).into())
    }
}

/// Errors raised here.
#[derive(Clone, Debug)]
enum MainError {
    /// No options were given through the CLI. At least one is needed to know which plugin to load.
    UnspecifiedOptions,
    /// There is no CLAP plugin in the bundle at this path.
    NoPluginInPath(PathBuf),
    /// No CLAP plugin with the given ID was found in the filesystem.
    NoPluginWithId(String),
    /// No CLAP plugin with the given ID was found in the bundle at the given path.
    NoPluginInPathWithId(PathBuf, String),
    /// There are multiple plugins in the given bundle, user needs to decide which one to load.
    MultiplePluginsInPath(PathBuf),
    /// There are multiple plugins with the given ID in the filesystem, user needs to decide which one to load.
    MultiplePluginsWithId(String),
}

impl Display for MainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self { MainError::UnspecifiedOptions => f.write_str("Please specify a plugin to load using the -p option or the -b option. Use --help for documentation."),
            MainError::NoPluginInPath(path) => write!(f,
                                                      "No plugins found in CLAP bundle at {}. Stopping.",
                                                      path.display()
            ),
            MainError::NoPluginWithId(id) => write!(f, "No plugins found matching id {id}. Stopping."),
            MainError::NoPluginInPathWithId(path, id) => write!(f,
                "Couldn't find a plugin matching id {id} in CLAP bundle at {}. Stopping.",
                path.display()),
            MainError::MultiplePluginsInPath(path) =>
                write!(f, "Found multiple plugins in CLAP bundle at {}. Specify a specific plugin ID using the -p option.", path.display()),
            MainError::MultiplePluginsWithId(id) => write!(f, "Found multiple plugins matching id {id}. Specify a specific bundle path using the -f option.")
        }
    }
}

impl Error for MainError {}
