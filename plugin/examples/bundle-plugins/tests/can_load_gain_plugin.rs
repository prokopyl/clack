use clack_host::prelude::*;
use std::env;
use std::path::Path;
use std::process::Command;

#[test]
pub fn can_load_gain_plugin() -> Result<(), Box<dyn std::error::Error>> {
    let project_root = project_root();
    run_bundle_plugins(project_root)?;
    let gain_plugin_path = project_root.join("target/dist/clack-plugin-gain.clap");

    let entry = unsafe { PluginEntry::load(gain_plugin_path)? };

    let desc = entry
        .get_plugin_factory()
        .unwrap()
        .plugin_descriptor(0)
        .unwrap();

    assert_eq!(desc.id(), Some(c"org.rust-audio.clack.gain"));

    Ok(())
}

fn project_root() -> &'static Path {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
}

fn run_bundle_plugins(project_root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cargo_path = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

    let status = Command::new(cargo_path)
        .current_dir(project_root)
        .args(["run", "-p", "bundle-plugins"])
        .status()?;

    if !status.success() {
        return Err("cargo build failed".into());
    }

    Ok(())
}
