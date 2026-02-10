use std::path::PathBuf;

/// Returns a list of all the standard CLAP search paths, per the CLAP specification.
///
/// Note that this function also takes the standard `CLAP_PATH` environment variable into
/// consideration.
pub fn standard_clap_paths() -> Vec<PathBuf> {
    let mut paths = vec![];

    // Standard env variable override.
    if let Some(env_var) = std::env::var_os("CLAP_PATH") {
        paths.extend(std::env::split_paths(&env_var))
    }

    // On Windows
    #[cfg(target_os = "windows")]
    {
        if let Some(val) = std::env::var_os("CommonProgramFiles") {
            paths.push(PathBuf::from(val).join("CLAP"))
        }

        if let Some(dir) = dirs_sys::known_folder_local_app_data() {
            paths.push(dir.join("Programs\\Common\\CLAP"));
        }
    }

    // On macOS
    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/Library/Audio/Plug-Ins/CLAP"));

        if let Some(home_dir) = dirs_sys::home_dir() {
            paths.push(home_dir.join("Library/Audio/Plug-Ins/CLAP"));
        }
    }

    // On Linux (and other UNIXes)
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    {
        paths.push("/usr/lib/clap".into());
        paths.push("/usr/lib64/clap".into());

        if let Some(home_dir) = dirs_sys::home_dir() {
            paths.push(home_dir.join(".clap"));
        }
    }

    paths
}
