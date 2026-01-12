use crate::preset_discovery::preset_data::*;
use crate::utils::cstr_from_nullable_ptr;
use clack_common::utils::ClapVersion;
use clap_sys::factory::preset_discovery::clap_preset_discovery_indexer;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

/// Various information about the host's indexer, provided at provider instantiation time.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct IndexerInfo<'a> {
    inner: NonNull<clap_preset_discovery_indexer>,
    _lifetime: PhantomData<&'a clap_preset_discovery_indexer>,
}

impl<'a> IndexerInfo<'a> {
    /// Creates a new [`IndexerInfo`] type from a given raw, C FFI compatible pointer.
    ///
    /// # Safety
    /// Pointer must be valid for the duration of the `'a` lifetime. Moreover, the contents of
    /// the `clap_preset_discovery_indexer` struct must all also be valid.
    #[inline]
    pub unsafe fn from_raw(inner: *const clap_preset_discovery_indexer) -> Option<Self> {
        Some(Self {
            inner: NonNull::new(inner.cast_mut())?,
            _lifetime: PhantomData,
        })
    }

    /// The [`ClapVersion`] the host uses.
    #[inline]
    pub const fn clap_version(&self) -> ClapVersion {
        ClapVersion::from_raw(self.to_raw().clap_version)
    }

    /// A user-friendly name for the host (e.g. "Bitwig Studio").
    ///
    /// This should always be set by the host.
    #[inline]
    pub const fn name(&self) -> &'a CStr {
        // SAFETY: this type ensures the pointers are valid
        match unsafe { cstr_from_nullable_ptr(self.to_raw().name) } {
            Some(name) => name,
            None => c"",
        }
    }

    /// The host's vendor (e.g. "Bitwig GmbH").
    ///
    /// This field is optional.
    #[inline]
    pub const fn version(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the pointers are valid
        unsafe { cstr_from_nullable_ptr(self.to_raw().version) }
    }

    /// A URL to the host's webpage.
    ///
    /// This field is optional.
    #[inline]
    pub const fn vendor(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the pointers are valid
        unsafe { cstr_from_nullable_ptr(self.to_raw().vendor) }
    }

    /// A version string for the host (e.g. "4.3").
    ///
    /// This should always be set by the host.
    #[inline]
    pub const fn url(&self) -> Option<&'a CStr> {
        // SAFETY: this type ensures the pointers are valid
        unsafe { cstr_from_nullable_ptr(self.to_raw().url) }
    }

    /// Returns the raw indexer handle as its C FFI-compatible struct.
    #[inline]
    const fn to_raw(self) -> clap_preset_discovery_indexer {
        // SAFETY: this type ensures the pointers are valid
        unsafe { self.inner.read() }
    }

    /// # Safety
    ///
    /// The resulting indexer must only be used within of after init()
    #[inline]
    pub(crate) unsafe fn to_indexer(self) -> Indexer<'a> {
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

/// A lightweight handle to the indexer.
#[repr(transparent)]
pub struct Indexer<'a> {
    inner: NonNull<clap_preset_discovery_indexer>,
    _lifetime: PhantomData<&'a clap_preset_discovery_indexer>,
}

impl<'a> Indexer<'a> {
    fn get(&self) -> clap_preset_discovery_indexer {
        // SAFETY: This type ensures the pointer is valid for reads
        unsafe { self.inner.read() }
    }

    /// Returns [information](IndexerInfo) about the indexer.
    #[inline]
    pub const fn info(&self) -> IndexerInfo<'a> {
        IndexerInfo {
            inner: self.inner,
            _lifetime: PhantomData,
        }
    }

    /// Declares a preset location for the host to index.
    ///
    /// # Errors
    /// This can return a [`IndexerError`] if the location is invalid, or if any other error occurred.
    #[inline]
    pub fn declare_location(&mut self, location: LocationInfo) -> Result<(), IndexerError> {
        let mut success = false;

        if let Some(declare_location) = self.get().declare_location {
            let location = location.to_raw();
            // SAFETY: This type ensures the indexer ptr is valid, strings come from &CStr and so are also valid
            success = unsafe { declare_location(self.inner.as_ptr(), &location) };
        }

        if success {
            Ok(())
        } else {
            Err(IndexerError { _inner: () })
        }
    }

    /// Declares a preset file type.
    ///
    /// # Errors
    ///
    /// This can return a [`IndexerError`] if the file type is invalid, or if any other error occurred.
    #[inline]
    pub fn declare_filetype(&mut self, file_type: FileType) -> Result<(), IndexerError> {
        let mut success = false;

        if let Some(declare_filetype) = self.get().declare_filetype {
            let filetype = file_type.to_raw();
            // SAFETY: This type ensures the indexer ptr is valid, strings come from &CStr and so are also valid
            success = unsafe { declare_filetype(self.inner.as_ptr(), &filetype) };
        }

        if success {
            Ok(())
        } else {
            Err(IndexerError { _inner: () })
        }
    }

    /// Declares a soundpack.
    ///
    /// # Errors
    /// This can return a [`IndexerError`] if the soundpack is invalid, or if any other error occurred.
    #[inline]
    pub fn declare_soundpack(&mut self, soundpack: Soundpack) -> Result<(), IndexerError> {
        let mut success = false;

        if let Some(declare_soundpack) = self.get().declare_soundpack {
            let soundpack = soundpack.to_raw();
            // SAFETY: This type ensures the indexer ptr is valid, strings come from &CStr and so are also valid
            success = unsafe { declare_soundpack(self.inner.as_ptr(), &soundpack) };
        }

        if success {
            Ok(())
        } else {
            Err(IndexerError { _inner: () })
        }
    }
}

impl Debug for Indexer<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.info().fmt(f)
    }
}

/// An error that can occur when using an [`Indexer`].
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct IndexerError {
    _inner: (),
}

impl Debug for IndexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("IndexerError")
    }
}

impl Display for IndexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Indexer Error")
    }
}

impl Error for IndexerError {}
