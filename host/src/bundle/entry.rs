use crate::bundle::PluginBundleError;
use clack_common::entry::EntryDescriptor;
use clack_common::utils::ClapVersion;
use std::ffi::CString;

pub struct LoadedEntry<'a> {
    entry: &'a EntryDescriptor,
}

impl<'a> LoadedEntry<'a> {
    /// # Safety
    ///
    /// User must ensure that the provided entry is fully valid, as well as everything it exposes.
    pub unsafe fn load(entry: &'a EntryDescriptor, path: &str) -> Result<Self, PluginBundleError> {
        let plugin_version = ClapVersion::from_raw(entry.clap_version);
        if !plugin_version.is_compatible() {
            return Err(PluginBundleError::IncompatibleClapVersion { plugin_version });
        }

        let path = CString::new(path).map_err(PluginBundleError::InvalidNulPath)?;

        if let Some(init) = entry.init {
            if !init(path.as_ptr()) {
                return Err(PluginBundleError::EntryInitFailed);
            }
        }

        Ok(Self { entry })
    }

    #[inline]
    pub fn entry(&self) -> &'a EntryDescriptor {
        self.entry
    }
}

impl<'a> Drop for LoadedEntry<'a> {
    fn drop(&mut self) {
        if let Some(deinit) = self.entry.deinit {
            // SAFETY: this type ensures deinit() is valid, and this can only be called once.
            unsafe { deinit() }
        }
    }
}

// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl<'a> Send for LoadedEntry<'a> {}
// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl<'a> Sync for LoadedEntry<'a> {}
