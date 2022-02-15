use crate::entry::{PluginEntry, PluginEntryError};
use clap_sys::entry::clap_plugin_entry;
use libloading::Library;
use std::ffi::OsStr;
use std::path::PathBuf;

pub struct PluginBundle {
    library: Library,
    path: PathBuf,
}

impl PluginBundle {
    pub fn load<P: AsRef<OsStr>>(path: P) -> Result<Self, PluginEntryError> {
        let path = path.as_ref();

        let library = unsafe { Library::new(path).map_err(PluginEntryError::LibraryLoadingError)? };

        Ok(Self {
            library,
            path: path.into(),
        })
    }

    pub fn get_entry(&self) -> Result<PluginEntry, PluginEntryError> {
        const SYMBOL_NAME: &[u8] = b"clap_entry\0";
        let symbol = unsafe { self.library.get::<*const clap_plugin_entry>(SYMBOL_NAME) }
            .map_err(PluginEntryError::LibraryLoadingError)?;

        let plugin_path = self
            .path
            .to_str()
            .ok_or(PluginEntryError::InvalidUtf8Path)
            .unwrap();

        Ok(unsafe { PluginEntry::from_raw(&**symbol, plugin_path)? })
    }
}
