use crate::bundle::PluginBundleError;
use clack_common::entry::EntryDescriptor;
use clack_common::utils::ClapVersion;
use std::ffi::CStr;
use std::ptr::NonNull;

pub struct LoadedEntry {
    entry: NonNull<EntryDescriptor>,
}

impl LoadedEntry {
    /// # Safety
    ///
    /// User must ensure that the provided entry is fully valid, as well as everything it exposes.
    /// User must also ensure this type *only* exists while `entry` is valid.
    pub unsafe fn load(entry: &EntryDescriptor, path: &CStr) -> Result<Self, PluginBundleError> {
        let plugin_version = ClapVersion::from_raw(entry.clap_version);
        if !plugin_version.is_compatible() {
            return Err(PluginBundleError::IncompatibleClapVersion { plugin_version });
        }

        if let Some(init) = entry.init {
            if !init(path.as_ptr()) {
                return Err(PluginBundleError::EntryInitFailed);
            }
        }

        Ok(Self {
            entry: entry.into(),
        })
    }

    #[inline]
    pub const fn entry(&self) -> &EntryDescriptor {
        // SAFETY: this type ensures entry is still valid.
        unsafe { self.entry.as_ref() }
    }
}

impl Drop for LoadedEntry {
    fn drop(&mut self) {
        // SAFETY: this type ensures entry is still valid.
        let entry = unsafe { self.entry.as_ref() };
        if let Some(deinit) = entry.deinit {
            // SAFETY: this type ensures deinit() is valid, and this can only be called once.
            unsafe { deinit() }
        }
    }
}

// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl Send for LoadedEntry {}
// SAFETY: Entries and factories are all thread-safe by the CLAP spec
unsafe impl Sync for LoadedEntry {}
