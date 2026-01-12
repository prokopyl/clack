use crate::discovery::*;
use crate::preset_discovery::get_presets;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod discovery;
mod preset_discovery;

/// A simple CLI tool to discover plugin bundles and extract information about their plugins and presets.
///
/// To inspect a specific plugin or bundle, at least one of the `--plugin-id` (`-p`) or the `--bundle-path` (`-b`) parameters should be used.
/// Otherwise, this tool will simply list all found CLAP bundles and plugins.
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

fn main() -> Result<(), ExitCode> {
    let args = Cli::parse();

    if matches!(
        args,
        Cli {
            bundle_path: None,
            plugin_id: None,
        }
    ) {
        list_plugins();
    } else {
        scan_bundle(args.bundle_path.as_deref(), args.plugin_id.as_deref())?;
    }

    Ok(())
}

/// Scans the CLAP standard paths for all the plugin descriptors that match the given ID.
pub fn list_plugins() {
    let standard_paths = standard_clap_paths();

    println!("Scanning the following directories for CLAP plugins:");
    for path in &standard_paths {
        println!("\t - {}", path.display())
    }

    let found_bundles = search_for_potential_bundles(&standard_paths);
    println!(" * Found {} potential CLAP bundles.", found_bundles.len());

    for bundle in scan_bundles(&found_bundles) {
        println!("  > At {}", bundle.path.to_string_lossy());
        for plugin in &bundle.plugins {
            println!("\t- {}", plugin)
        }
    }
}

pub fn scan_bundle(path: Option<&Path>, id: Option<&str>) -> Result<(), ExitCode> {
    let bundles = if let Some(path) = path {
        scan_plugin(path).into_iter().collect()
    } else {
        let standard_paths = scan_standard_paths();
        let bundles = search_for_potential_bundles(&standard_paths);
        scan_bundles_matching(&bundles, id.unwrap())
    };

    match bundles.as_slice() {
        [] => {
            eprintln!("No plugins found matching the given filters");
            return Err(ExitCode::FAILURE);
        }
        [bundle] => {
            println!("  > At {}", bundle.path.to_string_lossy());
            for preset in get_presets(&bundle.bundle) {
                for preset in preset.presets_per_location {
                    println!("{preset}")
                }

                if !preset.file_types.is_empty() {
                    println!(
                        "     > Filtered index on {} file types: ",
                        preset.file_types.len()
                    );

                    for file_type in &preset.file_types {
                        println!("    - {}", file_type);
                    }
                }

                if !preset.soundpacks.is_empty() {
                    println!(
                        "     > Registered {} sound packs: ",
                        preset.soundpacks.len()
                    );

                    for soundpack in &preset.soundpacks {
                        println!("    - {}", soundpack);
                    }
                }
            }
        }
        bundles => {
            for bundle in bundles {
                println!("  > At {}", bundle.path.to_string_lossy());
                for plugin in &bundle.plugins {
                    println!("\t- {}", plugin)
                }
            }
        }
    };

    Ok(())
}

fn scan_standard_paths() -> Vec<PathBuf> {
    let standard_paths = standard_clap_paths();

    println!("Scanning the following directories for CLAP plugins:");
    for path in &standard_paths {
        println!("\t - {}", path.display())
    }

    standard_paths
}
