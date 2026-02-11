use crate::bundle::PluginBundleError;
use crate::bundle::entry_provider::EntryProvider;
use clack_common::entry::EntryDescriptor;
use clack_common::utils::ClapVersion;
use std::ffi::CStr;
use std::ptr::NonNull;

pub struct LoadedEntry<E: EntryProvider>(E);

impl<E: EntryProvider> LoadedEntry<E> {
    pub fn load(entry_provider: E, bundle_path: &CStr) -> Result<Self, PluginBundleError> {
        let entry_pointer = entry_provider.entry_pointer();
        // SAFETY: TODO
        let entry = unsafe { entry_pointer.read() };
        let plugin_version = ClapVersion::from_raw(entry.clap_version);

        if !plugin_version.is_compatible() {
            return Err(PluginBundleError::IncompatibleClapVersion { plugin_version });
        }

        let Some(init) = entry.init else {
            return Err(PluginBundleError::IncompatibleClapVersion { plugin_version });
        };

        // SAFETY: TODO
        let result = unsafe { init(bundle_path.as_ptr()) };

        if !result {
            return Err(PluginBundleError::EntryInitFailed);
        }

        Ok(Self(entry_provider))
    }
}

impl<E: EntryProvider> Drop for LoadedEntry<E> {
    fn drop(&mut self) {
        let entry_pointer = self.entry_pointer();
        // SAFETY: TODO
        let entry = unsafe { entry_pointer.read() };

        // SAFETY: TODO, and this can only be called once.
        if let Some(deinit) = entry.deinit {
            unsafe { deinit() };
        }
    }
}

// TODO: unsafe
pub trait LoadedEntryDyn: Send + Sync + Send + 'static {
    fn entry_pointer(&self) -> NonNull<EntryDescriptor>;
}

impl<E: EntryProvider> LoadedEntryDyn for LoadedEntry<E> {
    #[inline]
    fn entry_pointer(&self) -> NonNull<EntryDescriptor> {
        self.0.entry_pointer()
    }
}
