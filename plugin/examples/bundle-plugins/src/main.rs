use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Plugin {
    project_name: &'static str,
}

const PLUGINS: &[Plugin] = &[Plugin {
    project_name: "clack-plugin-gain",
}];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_root = project_root();
    let cargo_path = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

    let status = Command::new(cargo_path)
        .current_dir(project_root)
        .args(["build", "--release"])
        .args(PLUGINS.iter().flat_map(|p| ["-p", p.project_name]))
        .status()?;

    if !status.success() {
        return Err("cargo build failed".into());
    }

    let target_dir = project_root.join("target/release");
    let dist_dir = project_root.join("target/dist");

    std::fs::create_dir_all(&dist_dir)?;

    for plugin in PLUGINS {
        let dylib_name = dylib_name(plugin.project_name);
        let dylib_path = target_dir.join(dylib_name);

        make_bundle(plugin, &dist_dir, &dylib_path)?;
    }

    Ok(())
}

fn project_root() -> &'static Path {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .unwrap()
}

fn dylib_name(project_name: &str) -> String {
    let project_name = project_name.replace('-', "_");

    #[cfg(target_os = "linux")]
    format!("lib{}.so", project_name)
}

fn make_bundle(
    plugin: &Plugin,
    dist_dir: &Path,
    dylib_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle_name = format!("{}.clap", plugin.project_name);
    let bundle_path = dist_dir.join(bundle_name);

    println!(
        "Copying {} to bundle {}",
        dylib_path.display(),
        bundle_path.display()
    );
    std::fs::copy(dylib_path, bundle_path)?;
    Ok(())
}
