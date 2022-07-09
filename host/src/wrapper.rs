use crate::host::{Host, HostError, HostMainThread, HostShared};
use crate::plugin::{PluginMainThreadHandle, PluginSharedHandle};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use selfie::Selfie;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::ptr::NonNull;

mod panic {
    #[cfg(not(test))]
    #[allow(unused)]
    pub use std::panic::catch_unwind;

    #[cfg(test)]
    #[inline]
    #[allow(unused)]
    pub fn catch_unwind<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
        f: F,
    ) -> std::thread::Result<R> {
        Ok(f())
    }
}

pub(crate) mod instance;
use instance::*;

pub(crate) mod descriptor;

pub(crate) mod data;
use data::*;

pub struct HostWrapper<H: for<'a> Host<'a>> {
    data: Selfie<'static, RawPluginInstanceRef, HostDataRef<H>>,
}

// SAFETY: The only non-thread-safe methods on this type are unsafe
unsafe impl<H: for<'h> Host<'h>> Send for HostWrapper<H> {}
unsafe impl<H: for<'h> Host<'h>> Sync for HostWrapper<H> {}

impl<H: for<'h> Host<'h>> HostWrapper<H> {
    pub(crate) fn new<FS, FH>(shared: FS, main_thread: FH) -> Self
    where
        FS: for<'s> FnOnce(&'s ()) -> <H as Host<'s>>::Shared,
        FH: for<'s> FnOnce(&'s <H as Host<'s>>::Shared) -> <H as Host<'s>>::MainThread,
    {
        let instance_ptr = Pin::new(RawPluginInstanceRef::default());

        Self {
            data: Selfie::new(instance_ptr, |_| {
                HostData::new(shared(&()), |s| main_thread(s))
            }),
        }
    }

    pub(crate) fn instantiated(&self, instance: *mut clap_plugin) {
        self.data.with_referential(|d| {
            // SAFETY: TODO?
            unsafe { d.shared().as_mut() }.instantiated(PluginSharedHandle::new(instance));
            unsafe { d.main_thread().as_mut() }.instantiated(PluginMainThreadHandle::new(instance));
        });
    }

    #[inline]
    pub(crate) unsafe fn activate<FA>(&self, audio_processor: FA)
    where
        FA: for<'a> FnOnce(
            &'a <H as Host<'a>>::Shared,
            &mut <H as Host<'a>>::MainThread,
        ) -> <H as Host<'a>>::AudioProcessor,
    {
        self.data.with_referential(|d| d.activate(audio_processor))
    }

    #[inline]
    pub(crate) unsafe fn deactivate(&self) {
        self.data.with_referential(|d| d.deactivate())
    }

    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        self.data.with_referential(|d| d.is_active())
    }

    /// Returns a raw, non-null pointer to the host's main thread ([`MainThread`](crate::host::Host::MainThread))
    /// struct.
    ///
    /// # Safety
    /// The caller must ensure this method is only called on the main thread.
    ///
    /// The pointer is safe to mutably dereference, as long as the caller ensures it is not being
    /// aliased, as per usual safety rules.
    #[inline]
    pub unsafe fn main_thread(&self) -> NonNull<<H as Host>::MainThread> {
        self.data.with_referential(|d| d.main_thread().cast())
    }

    /// Returns a raw, non-null pointer to the host's ([`AudioProcessor`](crate::host::Host::AudioProcessor))
    /// struct.
    ///
    /// # Safety
    /// The caller must ensure this method is only called on the audio thread.
    ///
    /// The pointer is safe to mutably dereference, as long as the caller ensures it is not being
    /// aliased, as per usual safety rules.
    #[inline]
    pub unsafe fn audio_processor(
        &self,
    ) -> Result<NonNull<<H as Host>::AudioProcessor>, HostError> {
        self.data.with_referential(|d| {
            d.audio_processor()
                .map(|a| a.cast())
                .ok_or(HostError::DeactivatedPlugin)
        })
    }

    #[inline]
    pub fn shared(&self) -> &<H as Host>::Shared {
        // SAFETY: TODO
        self.data
            .with_referential(|d| unsafe { d.shared().cast().as_ref() })
    }

    /// TODO: docs
    ///
    /// # Safety
    ///
    /// The given host wrapper type `H` **must** be the correct type for the received pointer. Otherwise,
    /// incorrect casts will occur, which will lead to Undefined Behavior.
    ///
    /// The `host` pointer must also point to a valid instance of `clap_host`, as created by
    /// the CLAP Host. While this function does a couple of simple safety checks, only a few common
    /// cases are actually covered (i.e. null checks), and those **must not** be relied upon for safety: those
    /// checks only exist to help debugging.
    pub unsafe fn handle<T, F>(host: *const clap_host, handler: F) -> Option<T>
    where
        F: FnOnce(&HostWrapper<H>) -> Result<T, HostWrapperError>,
    {
        match Self::handle_panic(host, handler) {
            Ok(value) => Some(value),
            Err(_e) => {
                // logging::plugin_log::<P>(host, &e); TODO

                None
            }
        }
    }

    unsafe fn handle_panic<T, F>(host: *const clap_host, handler: F) -> Result<T, HostWrapperError>
    where
        F: FnOnce(&HostWrapper<H>) -> Result<T, HostWrapperError>,
    {
        let plugin = Self::from_raw(host)?;

        panic::catch_unwind(AssertUnwindSafe(|| handler(plugin)))
            .map_err(|_| HostWrapperError::Panic)?
    }

    unsafe fn from_raw<'a>(raw: *const clap_host) -> Result<&'a Self, HostWrapperError> {
        raw.as_ref()
            .ok_or(HostWrapperError::NullHostInstance)?
            .host_data
            .cast::<HostWrapper<H>>()
            .as_ref()
            .ok_or(HostWrapperError::NullHostData)
    }
}

pub enum HostWrapperError {
    NullHostInstance,
    NullHostData,
    Panic,
    HostError(HostError),
}
