use crate::discovery::*;

mod discovery;
mod preset_discovery;

fn main() {
    list_plugins()
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
