use super::*;
use crate::utils::handle_panic;
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::ptr::NonNull;

#[repr(C)]
pub struct IndexerWrapper<I> {
    pub(crate) inner: I,
    _no_send: PhantomData<*const ()>,
}

impl<I> IndexerWrapper<I> {
    #[inline]
    pub(crate) fn new(inner: I) -> Pin<Box<Self>> {
        Box::pin(IndexerWrapper {
            inner,
            _no_send: PhantomData,
        })
    }

    #[inline]
    pub(crate) fn as_raw_mut(self: Pin<&mut Self>) -> *mut c_void {
        // SAFETY: this method does not move anything out, it just gets the pointer
        let s = unsafe { self.get_unchecked_mut() };

        s as *mut Self as *mut c_void
    }

    pub(crate) fn inner(&self) -> &I {
        &self.inner
    }

    pub(crate) fn inner_mut(self: Pin<&mut Self>) -> &mut I {
        // SAFETY: inner does not need to be pinned at all, only this struct needs to have a stable address.
        &mut unsafe { self.get_unchecked_mut() }.inner
    }

    unsafe fn from_raw<'a>(
        indexer: *const clap_preset_discovery_indexer,
    ) -> Result<&'a mut I, IndexerWrapperError> {
        let indexer =
            NonNull::new(indexer.cast_mut()).ok_or(IndexerWrapperError::NullIndexerPointer)?;

        // SAFETY: TODO
        let indexer = unsafe { indexer.read() };

        // SAFETY: TODO
        let wrapper = unsafe { indexer.indexer_data.cast::<Self>().as_mut() }
            .ok_or(IndexerWrapperError::NullIndexerDataPointer)?;

        Ok(&mut wrapper.inner)
    }

    #[inline]
    pub unsafe fn handle<T>(
        indexer: *const clap_preset_discovery_indexer,
        handler: impl FnOnce(&mut I) -> Result<T, IndexerWrapperError>,
    ) -> Option<T> {
        match Self::from_raw(indexer).and_then(|p| Self::handle_panic(p, handler)) {
            Ok(value) => Some(value),
            Err(e) => {
                eprintln!("{e}");

                None
            }
        }
    }

    #[inline]
    fn handle_panic<Pa, T, F>(parameter: Pa, handler: F) -> Result<T, IndexerWrapperError>
    where
        F: FnOnce(Pa) -> Result<T, IndexerWrapperError>,
    {
        handle_panic(AssertUnwindSafe(|| handler(parameter)))
            .map_err(|_| IndexerWrapperError::Panic)?
    }
}

pub enum IndexerWrapperError {
    NullIndexerPointer,
    NullIndexerDataPointer,
    Panic,
    /// An invalid parameter value was encountered.
    ///
    /// The given string may contain more information about which parameter was found to be invalid.
    InvalidParameter(&'static str),
}

impl Display for IndexerWrapperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexerWrapperError::NullIndexerPointer => f.write_str("Indexer pointer is null"),
            IndexerWrapperError::NullIndexerDataPointer => {
                f.write_str("Indexer data pointer is null")
            }
            IndexerWrapperError::Panic => f.write_str("Indexer panicked"),
            IndexerWrapperError::InvalidParameter(e) => {
                write!(f, "Invalid parameter to indexer function: {}", e)
            }
        }
    }
}
