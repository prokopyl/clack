use crate::bundle::entry::LoadedEntry;
use crate::bundle::PluginBundleError;
use clack_common::entry::EntryDescriptor;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

#[derive(Hash, Eq, PartialEq)]
struct EntryPointer(*const EntryDescriptor);

// SAFETY: we're treating those pointers as pure addresses, we never read from them
unsafe impl Send for EntryPointer {}

// SAFETY: we're treating those pointers as pure addresses, we never read from them
unsafe impl Sync for EntryPointer {}

static ENTRY_CACHE: LazyLock<Mutex<HashMap<EntryPointer, Arc<EntrySourceInner>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_or_insert(
    entry_pointer: EntryPointer,
    load_entry: impl FnOnce() -> Result<EntrySourceInner, PluginBundleError>,
) -> Result<CachedEntry, PluginBundleError> {
    let mut cache = ENTRY_CACHE.lock().unwrap_or_else(|e| e.into_inner());

    let s = match cache.entry(entry_pointer) {
        Entry::Occupied(e) => Arc::clone(e.get()),
        Entry::Vacant(e) => {
            let entry_source = Arc::new(load_entry()?);
            e.insert(Arc::clone(&entry_source));
            entry_source
        }
    };

    Ok(CachedEntry(Some(s)))
}

#[cfg(feature = "libloading")]
pub(crate) fn load_from_library(
    library: crate::bundle::library::PluginEntryLibrary,
    plugin_path: &str,
) -> Result<CachedEntry, PluginBundleError> {
    get_or_insert(EntryPointer(library.entry()), move || {
        // SAFETY: PluginEntryLibrary type guarantees the entry
        let entry = unsafe { LoadedEntry::load(library.entry(), plugin_path) }?;
        Ok(EntrySourceInner::FromLibrary {
            entry,
            _library: library,
        })
    })
}

/// # Safety
///
/// User must ensure that the provided entry is fully valid, as well as everything it exposes.
pub(crate) unsafe fn load_from_raw(
    entry_descriptor: &'static EntryDescriptor,
    plugin_path: &str,
) -> Result<CachedEntry, PluginBundleError> {
    get_or_insert(EntryPointer(entry_descriptor), || {
        // SAFETY: entry_descriptor is 'static, it is always valid.
        Ok(EntrySourceInner::FromRaw(LoadedEntry::load(
            entry_descriptor,
            plugin_path,
        )?))
    })
}

enum EntrySourceInner {
    FromRaw(LoadedEntry),
    #[cfg(feature = "libloading")]
    FromLibrary {
        // SAFETY: drop order is important! We must deinit the entry before unloading the library.
        entry: LoadedEntry,
        _library: crate::bundle::library::PluginEntryLibrary,
    },
}

#[derive(Clone)]
pub(crate) struct CachedEntry(Option<Arc<EntrySourceInner>>);

impl CachedEntry {
    #[inline]
    pub(crate) fn raw_entry(&self) -> &EntryDescriptor {
        let Some(entry) = &self.0 else {
            unreachable!("Unloaded state only exists during CachedEntry's Drop implementation")
        };

        match entry.as_ref() {
            EntrySourceInner::FromRaw(raw) => raw.entry(),
            #[cfg(feature = "libloading")]
            EntrySourceInner::FromLibrary { entry, .. } => entry.entry(),
        }
    }
}

impl Drop for CachedEntry {
    fn drop(&mut self) {
        let ptr = EntryPointer(self.raw_entry());

        // Drop the Arc. If it was the only one outside the cache, then its refcount should be 1.
        self.0 = None;

        let cache = ENTRY_CACHE.lock();

        let mut cache = cache.unwrap_or_else(|e| e.into_inner());

        if let Entry::Occupied(mut o) = cache.entry(ptr) {
            if Arc::get_mut(o.get_mut()).is_some() {
                o.remove();
            }
        }
    }
}
