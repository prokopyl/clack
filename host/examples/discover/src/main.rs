use crate::discovery::*;
use crate::preset_discovery::get_presets;
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

mod discovery;
mod preset_discovery;

/// A simple CLI tool to discover plugin files and extract information about their plugins and presets.
///
/// To inspect a specific plugin or file, at least one of the `--plugin-id` (`-p`) or the `--file-path` (`-f`) parameters should be used.
/// Otherwise, this tool will simply list all found CLAP file and plugins.
#[derive(Parser)]
#[command(about, long_about)]
struct Cli {
    /// Loads the plugin found in the CLAP file at the given path.
    ///
    /// If the file contains multiple plugins, this should be used in conjunction with the
    /// `--plugin-id` (`-p`) parameter to specify which one to load.
    #[arg(short = 'f', long = "file-path")]
    file_path: Option<PathBuf>,

    /// Loads the CLAP plugin with the given unique ID.
    ///
    /// This will start to scan the filesystem in the standard CLAP paths, and load all CLAP files
    /// found in those paths to search for the plugin matching the given ID.
    ///
    /// If multiple plugins matching the given ID were found on the filesystem, this should be used
    /// in conjunction with the `--file-path` (`-f`) parameter to specify which file to load the
    /// plugin from.
    #[arg(short = 'p', long = "plugin-id")]
    plugin_id: Option<String>,
}

fn main() -> Result<(), ExitCode> {
    let args = Cli::parse();

    if matches!(
        args,
        Cli {
            file_path: None,
            plugin_id: None,
        }
    ) {
        list_plugins();
    } else {
        scan_file(args.file_path, args.plugin_id.as_deref())?;
    }

    Ok(())
}

/// Scans the CLAP standard paths for all the plugin descriptors that match the given ID.
pub fn list_plugins() {
    let standard_paths = clack_finder::standard_clap_paths();

    println!("Scanning the following directories for CLAP plugins:");
    for path in &standard_paths {
        println!("\t - {}", path.display())
    }

    let potential_files = search_for_potential_files(standard_paths);
    println!(" * Found {} potential CLAP files.", potential_files.len());

    for clap_files in scan_files(potential_files) {
        println!("  > At {}", clap_files.path.to_string_lossy());
        for plugin in &clap_files.plugins {
            println!("\t- {}", plugin)
        }
    }
}

pub fn scan_file(path: Option<PathBuf>, id: Option<&str>) -> Result<(), ExitCode> {
    let files = if let Some(path) = path {
        scan_plugin_from_path(path).into_iter().collect()
    } else {
        let standard_paths = scan_standard_paths();
        let potential_files = search_for_potential_files(standard_paths);
        scan_files_matching(potential_files, id.unwrap())
    };

    match files.as_slice() {
        [] => {
            eprintln!("No plugins found matching the given filters");
            return Err(ExitCode::FAILURE);
        }
        [file] => {
            println!("  > At {}", file.path.to_string_lossy());
            for preset in get_presets(&file.entry) {
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
        files => {
            for file in files {
                println!("  > At {}", file.path.to_string_lossy());
                for plugin in &file.plugins {
                    println!("\t- {}", plugin)
                }
            }
        }
    };

    Ok(())
}

fn scan_standard_paths() -> Vec<PathBuf> {
    let standard_paths = clack_finder::standard_clap_paths();

    println!("Scanning the following directories for CLAP plugins:");
    for path in &standard_paths {
        println!("\t - {}", path.display())
    }

    standard_paths
}
