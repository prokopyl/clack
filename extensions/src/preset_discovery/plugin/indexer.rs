use crate::preset_discovery::preset_data::*;
use crate::utils::cstr_from_nullable_ptr;
use clap_sys::factory::preset_discovery::clap_preset_discovery_indexer;
use std::ffi::CStr;
use std::fmt::Debug;
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
    pub const fn name(&self) -> &'a CStr {
        // SAFETY: TODO
        match unsafe { cstr_from_nullable_ptr(self.to_raw().name) } {
            Some(name) => name,
            None => c"",
        }
    }

    #[inline]
    pub const fn version(&self) -> Option<&'a CStr> {
        // SAFETY: TODO
        unsafe { cstr_from_nullable_ptr(self.to_raw().version) }
    }

    #[inline]
    pub const fn vendor(&self) -> Option<&'a CStr> {
        // SAFETY: TODO
        unsafe { cstr_from_nullable_ptr(self.to_raw().vendor) }
    }

    #[inline]
    pub const fn url(&self) -> Option<&'a CStr> {
        // SAFETY: TODO
        unsafe { cstr_from_nullable_ptr(self.to_raw().url) }
    }

    #[inline]
    const fn to_raw(self) -> clap_preset_discovery_indexer {
        // SAFETY: TODO
        unsafe { self.inner.read() }
    }

    #[inline]
    pub(crate) unsafe fn to_indexer(&self) -> Indexer<'a> {
        Indexer {
            inner: self.inner,
            _lifetime: PhantomData,
        }
    }
}

impl Debug for IndexerInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexerInfo")
            .field("name", &self.name().to_string_lossy())
            .field("version", &self.version().map(CStr::to_string_lossy))
            .field("vendor", &self.vendor().map(CStr::to_string_lossy))
            .field("url", &self.url().map(CStr::to_string_lossy))
            .finish()
    }
}

#[repr(transparent)]
pub struct Indexer<'a> {
    inner: NonNull<clap_preset_discovery_indexer>,
    _lifetime: PhantomData<&'a clap_preset_discovery_indexer>,
}

impl<'a> Indexer<'a> {
    fn get(&self) -> clap_preset_discovery_indexer {
        // SAFETY: TODO
        unsafe { self.inner.read() }
    }

    #[inline]
    pub const fn info(&self) -> IndexerInfo<'a> {
        IndexerInfo {
            inner: self.inner,
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn declare_location(&mut self, location: LocationInfo) {
        if let Some(declare_location) = self.get().declare_location {
            let location = location.to_raw();
            // SAFETY: TODO
            unsafe { declare_location(self.inner.as_ptr(), &location) }; // TODO: error
        }
    }

    #[inline]
    pub fn declare_filetype(&mut self, file_type: FileType) {
        if let Some(declare_filetype) = self.get().declare_filetype {
            let filetype = file_type.to_raw();
            // SAFETY: TODO
            unsafe { declare_filetype(self.inner.as_ptr(), &filetype) };
        }
    }

    #[inline]
    pub fn declare_soundpack(&mut self, soundpack: Soundpack) {
        if let Some(declare_soundpack) = self.get().declare_soundpack {
            let soundpack = soundpack.to_raw();
            // SAFETY: TODO
            unsafe { declare_soundpack(self.inner.as_ptr(), &soundpack) };
        }
    }
}

impl Debug for Indexer<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.info().fmt(f)
    }
}
