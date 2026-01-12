use crate::utils::handle_panic;
use clack_host::extensions::prelude::HostWrapperError;
use clap_sys::factory::preset_discovery::*;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::ptr::NonNull;

/// A wrapper around an [indexer](super::IndexerImpl) implementation of type `I`.
///
/// This wrapper allows access to the indexer from an FFI pointer, while
/// also handling common FFI issues, such as error management and unwind safety.
///
/// This type is similar to [`HostWrapper`](clack_host::extensions::wrapper::HostWrapper), but for
/// an indexer instead of host handlers.
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

    /// # Safety
    ///
    /// `indexer` must come from `as_raw_mut`. The resulting reference must be unique.
    unsafe fn from_raw<'a>(
        indexer: *const clap_preset_discovery_indexer,
    ) -> Result<&'a mut I, HostWrapperError> {
        let indexer = NonNull::new(indexer.cast_mut()).ok_or(HostWrapperError::NullHostInstance)?;

        // SAFETY: Indexer pointer is valid for reads
        let indexer = unsafe { indexer.read() };

        // SAFETY: Indexer pointer comes from us, and is guaranteed by caller to be unique.
        let wrapper = unsafe { indexer.indexer_data.cast::<Self>().as_mut() }
            .ok_or(HostWrapperError::NullHostData)?;

        Ok(&mut wrapper.inner)
    }

    /// Provides a unique reference to the wrapped [indexer](super::IndexerImpl), to the given handler
    /// closure.
    ///
    /// Besides providing a reference, this function does a few extra safety checks:
    ///
    /// * The given `clap_preset_discovery_indexer` pointer is null-checked, as well as some other indexer-provided
    ///   pointers;
    /// * The handler is wrapped in [`std::panic::catch_unwind`];
    /// * Any [`HostWrapperError`] returned by the handler is caught.
    ///
    /// Note that some safety checks (e.g. the `clap_preset_discovery_indexer` pointer null-checks) may result in the
    /// closure never being called, and an error being returned only. Users of this function must
    /// not rely on the completion of this closure for safety, and must handle this function
    /// returning `None` gracefully.
    ///
    /// If all goes well, the return value of the handler closure is forwarded and returned by this
    /// function.
    ///
    /// # Errors
    /// If any safety check failed, or any error or panic occurred inside the handler closure, this
    /// function returns `None`.
    ///
    /// # Safety
    ///
    /// The given indexer type `I` **must** be the correct type for the received pointer. Otherwise,
    /// incorrect casts will occur, which will lead to Undefined Behavior.
    ///
    /// The `indexer` pointer must also point to a valid instance of `clap_preset_discovery_indexer`,
    /// as provided by the preset discovery provider. While this function does a couple of simple safety checks, only a few common
    /// cases are actually covered (i.e. null checks), and those **must not** be relied upon: those
    /// checks only exist to help debugging faulty hosts.
    #[inline]
    pub unsafe fn handle<T>(
        indexer: *const clap_preset_discovery_indexer,
        handler: impl FnOnce(&mut I) -> Result<T, HostWrapperError>,
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
    fn handle_panic<Pa, T, F>(parameter: Pa, handler: F) -> Result<T, HostWrapperError>
    where
        F: FnOnce(Pa) -> Result<T, HostWrapperError>,
    {
        handle_panic(AssertUnwindSafe(|| handler(parameter)))
            .map_err(|_| HostWrapperError::Panic)?
    }
}
