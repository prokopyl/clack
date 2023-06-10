mod discovery;
mod host;
mod stream;

use clap::Parser;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'f')]
    bundle_file: Option<PathBuf>,
    #[arg(short = 'p')]
    plugin_id: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match (&args.bundle_file, &args.plugin_id) {
        (Some(f), None) => run_from_path(f),
        (None, Some(p)) => run_from_id(p),
        (Some(f), Some(p)) => run_specific(f, p),
        (None, None) => Err(MainError::UnspecifiedOptions.into()),
    }
}

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

fn run_specific(path: &Path, id: &str) -> Result<(), Box<dyn Error>> {
    let bundle = discovery::load_plugin_id_from_path(path, id)?;

    if let Some(bundle) = bundle {
        host::run(bundle)
    } else {
        Err(MainError::NoPluginInPathWithId(path.to_path_buf(), id.to_string()).into())
    }
}

#[derive(Clone, Debug)]
enum MainError {
    UnspecifiedOptions,
    NoPluginInPath(PathBuf),
    NoPluginWithId(String),
    NoPluginInPathWithId(PathBuf, String),
    MultiplePluginsInPath(PathBuf),
    MultiplePluginsWithId(String),
}

impl Display for MainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self { MainError::UnspecifiedOptions => f.write_str("Please specify a plugin to load using the -p option or the -f option. Use --help for documentation."),
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
