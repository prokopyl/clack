use std::fs::FileType;
use std::path::Path;

pub fn may_be_clap_bundle(path: &Path, file_type: FileType) -> bool {
    #[cfg(target_os = "macos")]
    if file_type.is_file() {
        return false;
    }

    #[cfg(not(target_os = "macos"))]
    if file_type.is_dir() {
        return false;
    }

    let Some(ext) = path.extension() else {
        return false;
    };

    ext == "clap"
}
