use crate::preset_discovery::{Location, LocationData};
use clap_sys::factory::preset_discovery::clap_preset_discovery_indexer;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct IndexerInfo<'a> {
    inner: NonNull<clap_preset_discovery_indexer>,
    _lifetime: PhantomData<&'a clap_preset_discovery_indexer>,
}

impl<'a> IndexerInfo<'a> {
    #[inline]
    pub unsafe fn from_raw(inner: *const clap_preset_discovery_indexer) -> Option<Self> {
        Some(Self {
            inner: NonNull::new(inner.cast_mut())?,
            _lifetime: PhantomData,
        })
    }

    #[inline]
    pub(crate) unsafe fn to_indexer(&self) -> Indexer<'a> {
        Indexer {
            inner: self.inner,
            _lifetime: PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct Indexer<'a> {
    inner: NonNull<clap_preset_discovery_indexer>,
    _lifetime: PhantomData<&'a clap_preset_discovery_indexer>,
}

impl Indexer<'_> {
    fn get(&self) -> clap_preset_discovery_indexer {
        // SAFETY: TODO
        unsafe { self.inner.read() }
    }

    pub fn declare_location(&mut self, location: LocationData) {
        if let Some(declare_location) = self.get().declare_location {
            let location = location.to_raw();
            // SAFETY: TODO
            unsafe { declare_location(self.inner.as_ptr(), &location) }; // TODO: error
        }
    }
}
