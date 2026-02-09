use std::path::PathBuf;

/// Returns a list of all the standard CLAP search paths, per the CLAP specification.
pub fn standard_clap_paths() -> Vec<PathBuf> {
    let mut paths = vec![];

    #[cfg(target_os = "macos")]
    {

    }


    if let Some(home_dir) = dirs_sys::home_dir() {
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

    if let Some(env_var) = std::env::var_os("CLAP_PATH") {
        paths.extend(std::env::split_paths(&env_var))
    }

    paths
}
