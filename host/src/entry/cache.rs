use crate::entry::PluginEntryError;
use crate::entry::entry_provider::EntryProvider;
use crate::entry::loaded_entry::{LoadedEntry, LoadedEntryDyn};
use clack_common::entry::EntryDescriptor;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::CStr;
use std::hash::{BuildHasherDefault, DefaultHasher};
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

#[derive(Hash, Eq, PartialEq)]
struct EntryPointer(NonNull<EntryDescriptor>);

// SAFETY: we're treating those pointers as pure addresses, we never read from them
unsafe impl Send for EntryPointer {}

// SAFETY: we're treating those pointers as pure addresses, we never read from them
unsafe impl Sync for EntryPointer {}

static ENTRY_CACHE: Mutex<
    HashMap<EntryPointer, Arc<dyn LoadedEntryDyn>, BuildHasherDefault<DefaultHasher>>,
> = Mutex::new(HashMap::with_hasher(BuildHasherDefault::new()));

pub(crate) fn get_or_init<E: EntryProvider>(
    entry_provider: E,
    init_bundle_path: &CStr,
) -> Result<CachedEntry, PluginEntryError> {
    let mut cache = ENTRY_CACHE.lock().unwrap_or_else(|e| e.into_inner());

    let entry_pointer = EntryPointer(entry_provider.entry_pointer());

    let s = match cache.entry(entry_pointer) {
        Entry::Occupied(e) => Arc::clone(e.get()),
        Entry::Vacant(e) => {
            let entry_source: Arc<dyn LoadedEntryDyn> =
                Arc::new(LoadedEntry::load(entry_provider, init_bundle_path)?);
            e.insert(Arc::clone(&entry_source));
            entry_source
        }
    };

    Ok(CachedEntry(Some(s)))
}

#[derive(Clone)]
pub(crate) struct CachedEntry(Option<Arc<dyn LoadedEntryDyn>>);

impl CachedEntry {
    #[inline]
    pub(crate) fn raw_entry(&self) -> NonNull<EntryDescriptor> {
        let Some(entry) = &self.0 else {
            unreachable!("Unloaded state only exists during CachedEntry's Drop implementation")
        };

        entry.entry_pointer()
    }

    #[inline]
    pub fn as_ref(&self) -> &EntryDescriptor {
        // SAFETY: TODO
        unsafe { self.raw_entry().as_ref() }
    }

    #[inline]
    pub fn get(&self) -> EntryDescriptor {
        // SAFETY: TODO
        unsafe { self.raw_entry().read() }
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
