//! Helper utilities to help implementing the host side of custom CLAP extensions.

use crate::host::{Host, HostError, HostMainThread, HostShared};
use crate::plugin::{PluginAudioProcessorHandle, PluginMainThreadHandle, PluginSharedHandle};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
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

pub(crate) mod descriptor;

pub(crate) mod data;
use data::*;

pub struct HostWrapper<H: Host> {
    data: HostData<'static, H>,
}

// SAFETY: The only non-thread-safe methods on this type are unsafe
unsafe impl<H: Host> Send for HostWrapper<H> {}
unsafe impl<H: Host> Sync for HostWrapper<H> {}

impl<H: Host> HostWrapper<H> {
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

    /// Returns a raw, non-null pointer to the host's main thread ([`MainThread`](Host::MainThread))
    /// struct.
    ///
    /// # Safety
    /// The caller must ensure this method is only called on the main thread.
    ///
    /// The pointer is safe to mutably dereference, as long as the caller ensures it is not being
    /// aliased, as per usual safety rules.
    #[inline]
    pub unsafe fn main_thread(&self) -> NonNull<<H as Host>::MainThread<'_>> {
        self.data.main_thread()
    }

    /// Returns a raw, non-null pointer to the host's [`AudioProcessor`](Host::AudioProcessor)
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
    ) -> Result<NonNull<<H as Host>::AudioProcessor<'_>>, HostError> {
        self.data
            .inner
            .with_referential(|d| d.audio_processor().ok_or(HostError::DeactivatedPlugin))
    }

    /// Returns a shared reference to the host's [`Shared`](Host::Shared) struct.
    #[inline]
    pub fn shared(&self) -> &<H as Host>::Shared<'_> {
        unsafe { self.data.shared().cast().as_ref() }
    }

    pub(crate) fn new<FS, FH>(shared: FS, main_thread: FH) -> Pin<Box<Self>>
    where
        FS: for<'s> FnOnce(&'s ()) -> <H as Host>::Shared<'s>,
        FH: for<'s> FnOnce(&'s <H as Host>::Shared<'s>) -> <H as Host>::MainThread<'s>,
    {
        Box::pin(Self {
            data: HostData::new(shared(&()), |s| main_thread(s)),
        })
    }

    pub(crate) fn instantiated(&self, instance: *mut clap_plugin) {
        // SAFETY: TODO?
        unsafe { self.data.shared().as_ref() }.instantiated(PluginSharedHandle::new(instance));
        unsafe { self.data.main_thread().as_mut() }
            .instantiated(PluginMainThreadHandle::new(instance));
    }

    #[inline]
    pub(crate) unsafe fn activate<FA>(
        &mut self,
        audio_processor: FA,
        instance: *const clap_plugin,
    ) -> Result<(), HostError>
    where
        FA: for<'a> FnOnce(
            PluginAudioProcessorHandle<'a>,
            &'a <H as Host>::Shared<'a>,
            &mut <H as Host>::MainThread<'a>,
        ) -> <H as Host>::AudioProcessor<'a>,
    {
        //SAFETY: TODO
        self.data.inner.with_referential_mut(move |d|
            // SAFETY: TODO
            d.set_new_audio_processor(move |shared, main_thread| {
                let (handle, shared, main_thread) =
                    dumb_down_audio_processor_refs::<H>(instance, shared, main_thread);
                audio_processor(handle, shared, main_thread)
            }))
    }

    #[inline]
    pub(crate) fn deactivate<T>(
        &mut self,
        drop: impl for<'s> FnOnce(
            <H as Host>::AudioProcessor<'s>,
            &mut <H as Host>::MainThread<'s>,
        ) -> T,
    ) -> Result<T, HostError> {
        self.data.inner.with_referential_mut(|d| unsafe {
            let main_thread = d.main_thread().as_mut();
            Ok(drop(d.remove_audio_processor()?, main_thread))
        })
    }

    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        self.data.inner.with_referential(|d| d.is_active())
    }

    unsafe fn handle_panic<T, F>(host: *const clap_host, handler: F) -> Result<T, HostWrapperError>
    where
        F: FnOnce(&HostWrapper<H>) -> Result<T, HostWrapperError>,
    {
        let host = Self::from_raw(host)?;

        panic::catch_unwind(AssertUnwindSafe(|| handler(host)))
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum HostWrapperError {
    NullHostInstance,
    NullHostData,
    Panic,
    HostError(HostError),
}

impl From<HostError> for HostWrapperError {
    #[inline]
    fn from(e: HostError) -> Self {
        Self::HostError(e)
    }
}

fn dumb_down_audio_processor_refs<'instance, 'x, H: Host>(
    _instance: *const clap_plugin,
    _shared: &H::Shared<'instance>,
    _main_thread: &'x mut H::MainThread<'instance>,
) -> (
    PluginAudioProcessorHandle<'instance>,
    &'instance H::Shared<'instance>,
    &'x mut H::MainThread<'instance>,
) {
    todo!()
}
