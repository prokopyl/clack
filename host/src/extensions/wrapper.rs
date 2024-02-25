//! Helper utilities to help implementing the host side of custom CLAP extensions.

use crate::prelude::*;
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use std::cell::UnsafeCell;
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

pub struct HostWrapper<H: Host> {
    audio_processor: Option<UnsafeCell<<H as Host>::AudioProcessor<'static>>>,
    main_thread: Option<UnsafeCell<<H as Host>::MainThread<'static>>>,
    shared: Pin<Box<<H as Host>::Shared<'static>>>,
}

// SAFETY: The only non-thread-safe methods on this type are unsafe
unsafe impl<H: Host> Send for HostWrapper<H> {}
// SAFETY: The only non-thread-safe methods on this type are unsafe
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
        match Self::from_raw(host).and_then(|h| Self::handle_panic(handler, h)) {
            Ok(value) => Some(value),
            Err(_e) => {
                // logging::plugin_log::<P>(host, &e); TODO

                None
            }
        }
    }

    /// Returns a raw, non-null pointer to the host's ([`MainThread`](Host::MainThread)) struct.
    ///
    /// # Safety
    /// The caller must ensure this method is only called on the main thread.
    ///
    /// The pointer is safe to mutably dereference, as long as the caller ensures it is not being
    /// aliased, as per usual safety rules.
    #[inline]
    pub unsafe fn main_thread(&self) -> NonNull<<H as Host>::MainThread<'_>> {
        NonNull::new_unchecked(self.main_thread.as_ref().unwrap_unchecked().get()).cast()
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
        match &self.audio_processor {
            None => Err(HostError::DeactivatedPlugin),
            Some(ap) => Ok(NonNull::new_unchecked(ap.get()).cast()),
        }
    }

    /// Returns a shared reference to the host's [`Shared`](Host::Shared) struct.
    #[inline]
    pub fn shared(&self) -> &<H as Host>::Shared<'_> {
        // SAFETY: This type guarantees shared is never used mutably
        unsafe { shrink_shared_ref::<H>(&self.shared) }
    }

    pub(crate) fn new<FS, FH>(shared: FS, main_thread: FH) -> Pin<Box<Self>>
    where
        FS: for<'s> FnOnce(&'s ()) -> <H as Host>::Shared<'s>,
        FH: for<'s> FnOnce(&'s <H as Host>::Shared<'s>) -> <H as Host>::MainThread<'s>,
    {
        let mut wrapper = Box::pin(Self {
            audio_processor: None,
            main_thread: None,
            shared: Box::pin(shared(&())),
        });

        // Safety: we never move out of pinned_wrapper, we only update main_thread.
        let pinned_wrapper = unsafe { Pin::get_unchecked_mut(wrapper.as_mut()) };

        // SAFETY: This type guarantees main thread data cannot outlive shared
        pinned_wrapper.main_thread = Some(UnsafeCell::new(main_thread(unsafe {
            extend_shared_ref(&pinned_wrapper.shared)
        })));

        wrapper
    }

    /// # Safety
    /// This must only be called on the main thread. User must ensure the provided instance pointer
    /// is valid.
    pub(crate) unsafe fn instantiated(self: Pin<&mut Self>, instance: *mut clap_plugin) {
        // SAFETY: we only update the fields, we don't move them
        let pinned_self = unsafe { Pin::get_unchecked_mut(self) };

        // SAFETY: At this point there is no way main_thread could not have been set.
        unsafe { pinned_self.main_thread.as_mut().unwrap_unchecked() }
            .get_mut()
            .instantiated(PluginMainThreadHandle::new(instance));

        pinned_self
            .shared
            .instantiated(PluginSharedHandle::new(instance));
    }

    #[inline]
    pub(crate) fn setup_audio_processor<FA>(
        self: Pin<&mut Self>,
        audio_processor: FA,
        instance: *mut clap_plugin,
    ) -> Result<(), HostError>
    where
        FA: for<'a> FnOnce(
            PluginAudioProcessorHandle<'a>,
            &'a <H as Host>::Shared<'a>,
            &mut <H as Host>::MainThread<'a>,
        ) -> <H as Host>::AudioProcessor<'a>,
    {
        // SAFETY: we only update the fields, we don't move the struct
        let pinned_self = unsafe { Pin::get_unchecked_mut(self) };

        match &mut pinned_self.audio_processor {
            Some(_) => Err(HostError::AlreadyActivatedPlugin),
            None => {
                pinned_self.audio_processor = Some(UnsafeCell::new(audio_processor(
                    PluginAudioProcessorHandle::new(instance),
                    // SAFETY: Shared lives at least as long as the audio processor does.
                    unsafe { extend_shared_ref(&pinned_self.shared) },
                    // SAFETY: At this point there is no way main_thread could not have been set.
                    unsafe { pinned_self.main_thread.as_mut().unwrap_unchecked() }.get_mut(),
                )));
                Ok(())
            }
        }
    }

    #[inline]
    pub(crate) fn deactivate<T>(
        self: Pin<&mut Self>,
        drop: impl for<'s> FnOnce(
            <H as Host>::AudioProcessor<'s>,
            &mut <H as Host>::MainThread<'s>,
        ) -> T,
    ) -> Result<T, HostError> {
        // SAFETY: we only update the fields, we don't move the struct
        let pinned_self = unsafe { Pin::get_unchecked_mut(self) };

        match pinned_self.audio_processor.take() {
            None => Err(HostError::DeactivatedPlugin),
            Some(cell) => Ok(drop(
                cell.into_inner(),
                // SAFETY: At this point there is no way main_thread could not have been set.
                unsafe { pinned_self.main_thread.as_mut().unwrap_unchecked() }.get_mut(),
            )),
        }
    }

    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        self.audio_processor.is_some()
    }

    fn handle_panic<T, F, Pa>(handler: F, param: Pa) -> Result<T, HostWrapperError>
    where
        F: FnOnce(Pa) -> Result<T, HostWrapperError>,
    {
        panic::catch_unwind(AssertUnwindSafe(|| handler(param)))
            .map_err(|_| HostWrapperError::Panic)?
    }

    /// # Safety
    /// The host pointer must be valid (but can be null)
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

/// # Safety
/// The user MUST ensure the Shared ref lives long enough
unsafe fn extend_shared_ref<'a, H: HostShared<'a>>(shared: &H) -> &'a H {
    &*(shared as *const _)
}

/// # Safety
/// The user MUST prevent this reference to be written anywhere
unsafe fn shrink_shared_ref<'a, 'instance, H: Host>(
    shared: &'a H::Shared<'instance>,
) -> &'a H::Shared<'a> {
    let original_ptr = shared as *const H::Shared<'instance>;
    let transmuted_ptr: *const H::Shared<'a> = core::mem::transmute(original_ptr);

    &*transmuted_ptr
}
