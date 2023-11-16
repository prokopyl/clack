//! Helper utilities to help implementing the host side of custom CLAP extensions.

use crate::prelude::*;
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use selfie::refs::RefType;
use selfie::Selfie;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
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
    inner: Selfie<'static, Box<<H as Host>::Shared<'static>>, ReferentialHostDataRef<H>>,
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
        self.inner.with_referential(|d| d.main_thread().cast())
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
        self.inner
            .with_referential(|d| d.audio_processor().ok_or(HostError::DeactivatedPlugin))
    }

    /// Returns a shared reference to the host's [`Shared`](Host::Shared) struct.
    #[inline]
    pub fn shared(&self) -> &<H as Host>::Shared<'_> {
        // SAFETY: TODO
        unsafe { shrink_shared_ref::<H>(self.inner.owned()) }
    }

    pub(crate) fn new<FS, FH>(shared: FS, main_thread: FH) -> Pin<Box<Self>>
    where
        FS: for<'s> FnOnce(&'s ()) -> <H as Host>::Shared<'s>,
        FH: for<'s> FnOnce(&'s <H as Host>::Shared<'s>) -> <H as Host>::MainThread<'s>,
    {
        Box::pin(Self {
            inner: Selfie::new(Box::pin(shared(&())), |s| {
                // SAFETY: TODO
                let shared = unsafe { shrink_shared_ref::<H>(s) };
                ReferentialHostData::new(shared, main_thread(shared))
            }),
        })
    }

    pub(crate) fn instantiated(self: Pin<&mut Self>, instance: *mut clap_plugin) {
        // SAFETY: we only update the fields, we don't move them
        let pinned_self = unsafe { Pin::get_unchecked_mut(self) };

        pinned_self
            .inner
            .owned()
            .instantiated(PluginSharedHandle::new(instance));

        pinned_self.inner.with_referential_mut(|d| {
            d.main_thread
                .get_mut()
                .instantiated(PluginMainThreadHandle::new(instance))
        })
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

        pinned_self.inner.with_referential_mut(move |d| {
            d.set_new_audio_processor(move |shared, main_thread| {
                // SAFETY: shared is guaranteed to outlive all of its borrowers
                let shared = unsafe { extend_shared_ref(shared) };
                audio_processor(
                    PluginAudioProcessorHandle::new(instance),
                    shared,
                    main_thread,
                )
            })
        })
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

        pinned_self.inner.with_referential_mut(|d| unsafe {
            let main_thread = d.main_thread().as_mut();
            Ok(drop(d.remove_audio_processor()?, main_thread))
        })
    }

    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        self.inner.with_referential(|d| d.is_active())
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

struct ReferentialHostData<'shared, H: Host> {
    shared: &'shared H::Shared<'shared>,
    pub(crate) main_thread: UnsafeCell<H::MainThread<'shared>>,
    audio_processor: Option<UnsafeCell<H::AudioProcessor<'shared>>>,
}

impl<'shared, H: Host> ReferentialHostData<'shared, H> {
    #[inline]
    fn new(shared: &'shared H::Shared<'shared>, main_thread: H::MainThread<'shared>) -> Self {
        Self {
            shared,
            main_thread: UnsafeCell::new(main_thread),
            audio_processor: None,
        }
    }

    #[inline]
    fn main_thread(&self) -> NonNull<H::MainThread<'shared>> {
        // SAFETY: the pointer comes from UnsafeCell, it cannot be null
        unsafe { NonNull::new_unchecked(self.main_thread.get().cast()) }
    }

    #[inline]
    fn audio_processor(&self) -> Option<NonNull<H::AudioProcessor<'_>>> {
        self.audio_processor
            .as_ref()
            // SAFETY: pointer cannot be null as it comes from ce cell
            .map(|cell| unsafe { NonNull::new_unchecked(cell.get().cast()) })
    }

    #[inline]
    fn is_active(&self) -> bool {
        self.audio_processor.is_some()
    }

    fn set_new_audio_processor<
        FA: FnOnce(
            &'shared H::Shared<'shared>,
            &mut H::MainThread<'shared>,
        ) -> H::AudioProcessor<'shared>,
    >(
        &mut self,
        audio_processor: FA,
    ) -> Result<(), HostError> {
        match &mut self.audio_processor {
            Some(_) => Err(HostError::AlreadyActivatedPlugin),
            None => {
                self.audio_processor = Some(UnsafeCell::new(audio_processor(
                    self.shared,
                    self.main_thread.get_mut(),
                )));
                Ok(())
            }
        }
    }

    fn remove_audio_processor(&mut self) -> Result<H::AudioProcessor<'shared>, HostError> {
        self.audio_processor
            .take()
            .map(|cell| cell.into_inner())
            .ok_or(HostError::DeactivatedPlugin)
    }
}

struct ReferentialHostDataRef<H>(PhantomData<H>);

impl<'shared, H: Host> RefType<'shared> for ReferentialHostDataRef<H> {
    type Ref = ReferentialHostData<'shared, H>;
}
