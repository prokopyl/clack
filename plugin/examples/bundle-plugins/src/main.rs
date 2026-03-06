use std::env;
use std::path::Path;
use std::process::Command;

#[allow(unused)]
struct Plugin {
    project_name: &'static str,
    id: &'static str,
    display_name: &'static str,
}

const PLUGINS: &[Plugin] = &[
    Plugin {
        project_name: "clack-plugin-gain",
        id: "org.rust-audio.clack.gain",
        display_name: "Clack Gain Example",
    },
    Plugin {
        project_name: "clack-plugin-gain-gui",
        id: "org.rust-audio.clack.gain-gui",
        display_name: "Clack Gain GUI Example",
    },
    Plugin {
        project_name: "clack-plugin-gain-presets",
        id: "org.rust-audio.clack.gain-presets",
        display_name: "Clack Gain Presets Example",
    },
    Plugin {
        project_name: "clack-plugin-polysynth",
        id: "org.rust-audio.clack.polysynth",
        display_name: "Clack PolySynth Example",
    },
];

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
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

        #[cfg(not(target_os = "macos"))]
        make_bundle(plugin, &dist_dir, &dylib_path)?;
        #[cfg(target_os = "macos")]
        make_macos_bundle(plugin, &dist_dir, &dylib_path)?;
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
    {
        format!("lib{}.so", project_name)
    }

    #[cfg(target_os = "macos")]
    {
        format!("lib{}.dylib", project_name)
    }

    #[cfg(target_os = "windows")]
    {
        format!("{}.dll", project_name)
    }
}

#[cfg(not(target_os = "macos"))]
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

#[cfg(target_os = "macos")]
fn make_macos_bundle(
    plugin: &Plugin,
    dist_dir: &Path,
    dylib_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let bundle_name = format!("{}.clap", plugin.project_name);
    let bundle_path = dist_dir.join(bundle_name);

    remove_bundle_if_exists(&bundle_path);

    let executable_dir = bundle_path.join("Contents/MacOS");
    let executable_path = executable_dir.join(plugin.project_name);

    println!("Crating bundle {}", bundle_path.display());
    std::fs::create_dir_all(executable_dir)?;

    println!(
        "Copying {} to bundle {}",
        dylib_path.display(),
        executable_path.display()
    );
    std::fs::copy(dylib_path, executable_path)?;

    let plist = info_plist(plugin);
    let plist_path = bundle_path.join("Contents/Info.plist");
    std::fs::write(plist_path, plist)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn remove_bundle_if_exists(bundle_path: &Path) {
    let Ok(existing) = std::fs::metadata(bundle_path) else {
        return;
    };

    let result = if existing.is_dir() {
        std::fs::remove_dir_all(bundle_path)
    } else {
        std::fs::remove_file(bundle_path)
    };

    match result {
        Ok(()) => {
            println!("Removed previous bundle {}", bundle_path.display());
        }
        Err(e) => eprintln!("Failed to remove {}: {}", bundle_path.display(), e),
    }
}

#[cfg(target_os = "macos")]
fn info_plist(plugin: &Plugin) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>{}</string>
</dict>
</plist>"#,
        plugin.display_name, plugin.project_name, plugin.id
    )
}
