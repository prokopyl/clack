use crate::entry::PluginEntry;
use clap_sys::entry::clap_plugin_entry;
use libloading::Library;
use std::error::Error;
use std::ffi::OsStr;
use std::path::PathBuf;

pub struct PluginBundle {
    library: Library,
    path: PathBuf,
}

impl PluginBundle {
    // TODO: errors
    pub fn load<P: AsRef<OsStr>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();

        Ok(Self {
            library: unsafe { Library::new(path)? },
            path: path.into(),
        })
    }

    pub fn get_entry(&self) -> Result<PluginEntry, Box<dyn Error>> {
        const SYMBOL_NAME: &[u8] = b"clap_entry\0";
        let symbol = unsafe { self.library.get::<*const clap_plugin_entry>(SYMBOL_NAME) }?;
        Ok(unsafe { PluginEntry::from_raw(&**symbol, self.path.to_str().unwrap())? })
        // TODO: OsStr unwrap
    }
}
