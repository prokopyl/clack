use crate::bundle::entry::LoadedEntry;
use crate::bundle::library::PluginEntryLibrary;
use crate::bundle::PluginBundleError;
use clack_common::entry::EntryDescriptor;
use selfie::refs::RefType;
use selfie::Selfie;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Hash, Eq, PartialEq)]
struct EntryPointer(*const EntryDescriptor);

unsafe impl Send for EntryPointer {}
unsafe impl Sync for EntryPointer {}

static ENTRY_CACHE: OnceLock<Mutex<HashMap<EntryPointer, Arc<EntrySourceInner>>>> = OnceLock::new();

fn get_or_insert(
    entry_pointer: EntryPointer,
    load_entry: impl FnOnce() -> Result<EntrySourceInner, PluginBundleError>,
) -> Result<CachedEntry, PluginBundleError> {
    let cache = ENTRY_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock();

    let mut cache = match cache {
        Ok(guard) => guard,
        Err(e) => e.into_inner(),
    };

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

pub(crate) unsafe fn load_from_library(
    library: PluginEntryLibrary,
    plugin_path: &str,
) -> Result<CachedEntry, PluginBundleError> {
    get_or_insert(EntryPointer(library.entry()), move || {
        Ok(EntrySourceInner::FromLibrary(
            Selfie::try_new(Pin::new(library), |entry| unsafe {
                LoadedEntry::load(entry, plugin_path)
            })
            // The library can be discarded completely
            .map_err(|e| e.error)?,
        ))
    })
}

pub(crate) unsafe fn load_from_raw(
    entry_descriptor: &'static EntryDescriptor,
    plugin_path: &str,
) -> Result<CachedEntry, PluginBundleError> {
    get_or_insert(EntryPointer(entry_descriptor), || {
        Ok(EntrySourceInner::FromRaw(LoadedEntry::load(
            entry_descriptor,
            plugin_path,
        )?))
    })
}

pub(crate) struct LoadedEntryRef;

impl<'a> RefType<'a> for LoadedEntryRef {
    type Ref = LoadedEntry<'a>;
}

enum EntrySourceInner {
    FromRaw(LoadedEntry<'static>),
    FromLibrary(Selfie<'static, PluginEntryLibrary, LoadedEntryRef>),
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
            EntrySourceInner::FromLibrary(bundle) => bundle.with_referential(|e| e.entry()),
        }
    }
}

impl Drop for CachedEntry {
    fn drop(&mut self) {
        let ptr = EntryPointer(self.raw_entry());

        // Drop the Arc. If it was the only one outside of the cache, then its refcount should be 1.
        self.0 = None;

        let cache = ENTRY_CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock();

        let mut cache = match cache {
            Ok(guard) => guard,
            Err(e) => e.into_inner(),
        };

        if let Entry::Occupied(mut o) = cache.entry(ptr) {
            if Arc::get_mut(o.get_mut()).is_some() {
                o.remove();
            }
        }
    }
}
