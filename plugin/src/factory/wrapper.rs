#![deny(missing_docs)]

use crate::extensions::wrapper::handle_panic;
use crate::factory::error::FactoryWrapperError;
use clack_common::factory::RawFactoryPointer;
use std::panic::AssertUnwindSafe;
use std::ptr::NonNull;

/// A wrapper around a `clack` factory of a given `F` type, as well as its `Raw` CLAP representation.
///
/// This wrapper allows to safely create a CLAP-compatible factory pointer, as well as soundly
/// casting that pointer back to this wrapper's type, allowing to access the `F` type using the
/// [`handle`](FactoryWrapper::handle) function.
#[repr(C)]
pub struct FactoryWrapper<Raw, F> {
    raw: Raw,
    inner: F,
}

impl<Raw, F> FactoryWrapper<Raw, F> {
    /// Creates a new factory wrapper from a `Raw` CLAP type instance and an associated instance of `F`.
    #[inline]
    pub const fn new(raw: Raw, inner: F) -> Self {
        Self { raw, inner }
    }

    /// Returns a raw factory pointer compatible with this wrapper's `Raw` CLAP representation
    #[inline]
    pub fn as_raw(&self) -> RawFactoryPointer<'_, Raw> {
        // Note that we borrow all of `Self` here, not just the raw field.
        // This keeps the borrow alive and valid for the whole lifetime of the resulting factory pointer,
        // which means that when casting back to `Self` we can still soundly access all of `F` as well.
        let self_ptr: NonNull<Self> = self.into();

        // SAFETY: This pointer comes from a reference, so it is always valid for the matching lifetime
        unsafe { RawFactoryPointer::from_raw(self_ptr.cast()) }
    }

    /// Returns a raw pointer to the `Raw` CLAP representation contained in this wrapper.
    #[inline]
    pub fn as_raw_ptr(&self) -> *const Raw {
        self.as_raw().as_raw().as_ptr()
    }

    /// Returns a shared reference to the `F` factory instance.
    #[inline]
    pub const fn factory(&self) -> &F {
        &self.inner
    }

    /// Provides a shared reference to the `F` factory instance to the given handler closure.
    ///
    /// Besides providing a reference, this function does a few extra safety checks:
    ///
    /// * The given `Raw` factory pointer is null-checked;
    /// * The handler is wrapped in [`std::panic::catch_unwind`];
    /// * Any [`FactoryWrapperError`] returned by the handler is caught.
    ///
    /// If any of the above safety check fails, an error message is logged through stderr (as CLAP
    /// logging facilities are not available at the factory stage).
    ///
    /// Note that some safety checks (e.g. the `Raw` pointer null-checks) may result in the
    /// closure never being called, and an error being returned only. Users of this function must
    /// not rely on the completion of this closure for safety, and must handle this function
    /// returning `None` gracefully.
    ///
    /// If all goes well, the return value of the handler closure is forwarded and returned by this
    /// function.
    ///
    /// # Errors
    ///
    /// If any safety check failed, or any error or panic occurred inside the handler closure, this
    /// function returns `None`, and the error message is logged.
    ///
    /// # Safety
    ///
    /// The `raw` pointer must be created by the [`as_raw`](Self::as_raw) method, from an instance that
    /// is still valid for the duration of the `handler` call.
    ///
    /// While this function does a couple of simple safety checks, only a few common
    /// cases are actually covered (i.e. null checks), and those **must not** be relied upon: those
    /// checks only exist to help debugging faulty hosts.
    ///
    /// # Example
    ///
    /// This is the implementation of the [`plugin_count`](crate::factory::plugin::PluginFactoryImpl::plugin_count)
    /// callback's C wrapper.
    ///
    /// ```
    /// use clap_sys::factory::plugin_factory::clap_plugin_factory;
    /// use clack_plugin::plugin::{Plugin, PluginMainThread};
    /// use clack_plugin::factory::{FactoryWrapper, plugin::PluginFactoryImpl};
    ///
    /// unsafe extern "C" fn get_plugin_count<F: PluginFactoryImpl>(factory: *const clap_plugin_factory) -> u32 {
    ///    FactoryWrapper::<clap_plugin_factory, F>::handle(factory, |factory| {
    ///        Ok(factory.plugin_count())
    ///    })
    ///    .unwrap_or(0)
    /// }
    /// ```
    #[must_use]
    pub unsafe fn handle<T>(
        raw: *const Raw,
        handler: impl FnOnce(&F) -> Result<T, FactoryWrapperError>,
    ) -> Option<T> {
        // SAFETY: The caller ensures this pointer comes from the reference in as_raw
        let factory = unsafe { raw.cast::<Self>().as_ref() };
        let result = factory.ok_or(FactoryWrapperError::NullFactoryInstance);

        let result = result.and_then(|f| f.handle_panic(handler));

        Self::handle_result(result)
    }

    #[inline]
    fn handle_panic<T>(
        &self,
        handler: impl FnOnce(&F) -> Result<T, FactoryWrapperError>,
    ) -> Result<T, FactoryWrapperError> {
        match handle_panic(AssertUnwindSafe(|| handler(&self.inner))) {
            Err(_) => Err(FactoryWrapperError::Panic),
            Ok(Err(e)) => Err(e),
            Ok(Ok(val)) => Ok(val),
        }
    }

    #[inline]
    fn handle_result<T>(result: Result<T, FactoryWrapperError>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(e) => {
                eprintln!("[CLAP_PLUGIN_FACTORY_ERROR] {e}");

                None
            }
        }
    }
}
