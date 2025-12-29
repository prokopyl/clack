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
    pub(crate) unsafe fn to_indexer(&self) -> Indexer<'a> {
        Indexer {
            inner: self.inner,
            _lifetime: PhantomData,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Indexer<'a> {
    inner: NonNull<clap_preset_discovery_indexer>,
    _lifetime: PhantomData<&'a clap_preset_discovery_indexer>,
}
