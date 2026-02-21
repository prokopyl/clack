use crate::entry::PluginEntryError;
use crate::entry::entry_provider::EntryProvider;
use clack_common::entry::EntryDescriptor;
use clack_common::utils::ClapVersion;
use std::ffi::CStr;
use std::ptr::NonNull;

pub struct LoadedEntry<E: EntryProvider>(E);

#[inline]
fn get_entry(e: &impl EntryProvider) -> EntryDescriptor {
    // SAFETY: Safety contract of EntryProvider enforces that this pointer is always valid for reads.
    unsafe { e.entry_pointer().read() }
}

impl<E: EntryProvider> LoadedEntry<E> {
    pub fn load(entry_provider: E, bundle_path: &CStr) -> Result<Self, PluginEntryError> {
        let entry = get_entry(&entry_provider);

        let plugin_version = ClapVersion::from_raw(entry.clap_version);

        if !plugin_version.is_compatible() {
            return Err(PluginEntryError::IncompatibleClapVersion { plugin_version });
        }

        let Some(init) = entry.init else {
            return Err(PluginEntryError::IncompatibleClapVersion { plugin_version });
        };

        // SAFETY: Provided pointer comes from a valid C string.
        let result = unsafe { init(bundle_path.as_ptr()) };

        if !result {
            return Err(PluginEntryError::EntryInitFailed);
        }

        Ok(Self(entry_provider))
    }
}

impl<E: EntryProvider> Drop for LoadedEntry<E> {
    fn drop(&mut self) {
        let entry = get_entry(&self.0);

        if let Some(deinit) = entry.deinit {
            // SAFETY: This is only called this here, so because this is in Drop, this cannot be
            // called twice.
            // It is also necessarily called after a successful init, as checked by load() which is
            // the only constructor of this type
            unsafe { deinit() };
        }
    }
}

/// # Safety
///
/// Same as [`EntryProvider`]
pub unsafe trait LoadedEntryDyn: Send + Sync + Send + 'static {
    fn entry_pointer(&self) -> NonNull<EntryDescriptor>;
}

// SAFETY: same as EntryProvider
unsafe impl<E: EntryProvider> LoadedEntryDyn for LoadedEntry<E> {
    #[inline]
    fn entry_pointer(&self) -> NonNull<EntryDescriptor> {
        self.0.entry_pointer()
    }
}
