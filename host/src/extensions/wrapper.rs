//! Helper utilities to help implementing the host side of custom CLAP extensions.

use crate::plugin::DestroyLock;
use crate::prelude::*;
use crate::util::UnsafeOptionCell;
use clap_sys::ext::log::{
    clap_log_severity, CLAP_LOG_HOST_MISBEHAVING, CLAP_LOG_PLUGIN_MISBEHAVING,
};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Once, OnceLock};

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
mod logging;

// Safety note: once this type is constructed, a pointer to it will be given to the plugin instance,
// which means we can never
pub struct HostWrapper<H: HostHandlers> {
    audio_processor: UnsafeOptionCell<<H as HostHandlers>::AudioProcessor<'static>>,
    main_thread: UnsafeOptionCell<<H as HostHandlers>::MainThread<'static>>,
    shared: Pin<Box<<H as HostHandlers>::Shared<'static>>>,

    // Init stuff
    init_guard: Once,
    init_started: AtomicBool,
    plugin_ptr: OnceLock<NonNull<clap_plugin>>,

    // Drop stuff
    destroy_lock: Arc<DestroyLock>,
}

// SAFETY: The only non-thread-safe methods on this type are unsafe
unsafe impl<H: HostHandlers> Send for HostWrapper<H> {}
// SAFETY: The only non-thread-safe methods on this type are unsafe
unsafe impl<H: HostHandlers> Sync for HostWrapper<H> {}

impl<H: HostHandlers> HostWrapper<H> {
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
        let result = Self::from_raw(host).and_then(|h| {
            Self::handle_panic(h, |h| {
                h.ensure_initializing_called();
                handler(h)
            })
        });

        match result {
            Ok(value) => Some(value),
            Err(e) => {
                logging::host_log(host, &e);

                None
            }
        }
    }

    /// Returns a raw, non-null pointer to the host's ([`MainThread`](HostHandlers::MainThread)) struct.
    ///
    /// # Safety
    /// The caller must ensure this method is only called on the main thread.
    ///
    /// The pointer is safe to mutably dereference, as long as the caller ensures it is not being
    /// aliased, as per usual safety rules.
    #[inline]
    pub unsafe fn main_thread(&self) -> NonNull<<H as HostHandlers>::MainThread<'_>> {
        self.main_thread.as_ptr_unchecked().cast()
    }

    /// Returns a raw, non-null pointer to the host's [`AudioProcessor`](HostHandlers::AudioProcessor)
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
    ) -> Result<NonNull<<H as HostHandlers>::AudioProcessor<'_>>, HostError> {
        let ptr = self
            .audio_processor
            .as_ptr()
            .ok_or(HostError::DeactivatedPlugin)?;

        Ok(ptr.cast())
    }

    /// Returns a shared reference to the host's [`Shared`](HostHandlers::Shared) struct.
    #[inline]
    pub fn shared(&self) -> &<H as HostHandlers>::Shared<'_> {
        // SAFETY: This type guarantees shared is never used mutably
        unsafe { shrink_shared_ref::<H>(&self.shared) }
    }

    pub(crate) fn new<FS, FH>(shared: FS, main_thread: FH) -> Pin<Arc<Self>>
    where
        FS: for<'s> FnOnce(&'s ()) -> <H as HostHandlers>::Shared<'s>,
        FH: for<'s> FnOnce(
            &'s <H as HostHandlers>::Shared<'s>,
        ) -> <H as HostHandlers>::MainThread<'s>,
    {
        // We use Arc only because Box<T> implies Unique<T>, which is not the case since the plugin
        // will effectively hold a shared pointer to this.
        let mut wrapper = Arc::new(Self {
            audio_processor: UnsafeOptionCell::new(),
            main_thread: UnsafeOptionCell::new(),
            shared: Box::pin(shared(&())),
            init_guard: Once::new(),
            init_started: AtomicBool::new(false),
            plugin_ptr: OnceLock::new(),
            destroy_lock: Arc::new(DestroyLock::new()),
        });

        // PANIC: we have the only Arc copy of this wrapper data.
        let wrapper_mut = Arc::get_mut(&mut wrapper).unwrap();

        // SAFETY: This type guarantees main thread data cannot outlive shared
        unsafe {
            wrapper_mut
                .main_thread
                .put(main_thread(extend_shared_ref(&wrapper_mut.shared)));
        }

        // SAFETY: wrapper is the only reference to the data, we can guarantee it will remain pinned
        // until drop happens.
        unsafe { Pin::new_unchecked(wrapper) }
    }

    /// # Safety
    /// The pointer and the plugin instance it points to must remain valid for the lifetime of this
    /// wrapper.
    pub(crate) unsafe fn created(&self, instance: NonNull<clap_plugin>) {
        let _ = self.plugin_ptr.set(instance);
    }

    /// # Safety
    /// This must only be called on the main thread. User must ensure the provided instance pointer
    /// is valid.
    pub(crate) unsafe fn instantiated(&self) {
        self.ensure_initializing_called();
        let instance = *self.plugin_ptr.get().unwrap();

        // SAFETY: At this point there is no way main_thread could not have been set.
        self.main_thread()
            .as_mut()
            .initialized(InitializedPluginHandle::new(
                self.destroy_lock.clone(),
                instance,
            ));
    }

    // TODO: bikeshed
    pub(crate) fn start_instance_destroy(&self) {
        self.destroy_lock.start_destroying();
    }

    /// # Safety
    /// The user must ensure this is only called on the main thread, and not concurrently
    /// to any other main-thread OR audio-thread method.
    #[inline]
    pub(crate) unsafe fn setup_audio_processor<FA>(
        &self,
        audio_processor: FA,
    ) -> Result<(), HostError>
    where
        FA: for<'a> FnOnce(
            &'a <H as HostHandlers>::Shared<'a>,
            &mut <H as HostHandlers>::MainThread<'a>,
        ) -> <H as HostHandlers>::AudioProcessor<'a>,
    {
        if self.audio_processor.is_some() {
            return Err(HostError::AlreadyActivatedPlugin);
        }

        self.audio_processor.put(audio_processor(
            // SAFETY: Shared lives at least as long as the audio processor does.
            unsafe { extend_shared_ref(&self.shared) },
            // SAFETY: The user enforces that this is only called on the main thread, and
            // non-concurrently to any other main-thread method.
            unsafe { self.main_thread().cast().as_mut() },
        ));
        Ok(())
    }

    /// # Safety
    /// The user must ensure this is only called on the main thread, and not concurrently
    /// to any other main-thread OR audio-thread method.
    #[inline]
    pub(crate) unsafe fn teardown_audio_processor<T>(
        &self,
        drop: impl for<'s> FnOnce(
            <H as HostHandlers>::AudioProcessor<'s>,
            &mut <H as HostHandlers>::MainThread<'s>,
        ) -> T,
    ) -> Result<T, HostError> {
        // SAFETY: The user enforces that this is called and non-concurrently to any other audio-thread method.
        match self.audio_processor.take() {
            None => Err(HostError::DeactivatedPlugin),
            Some(audio_processor) => Ok(drop(
                audio_processor,
                // SAFETY: The user enforces that this is only called on the main thread, and
                // non-concurrently to any other main-thread method.
                unsafe { self.main_thread().cast().as_mut() },
            )),
        }
    }

    /// # Safety
    /// the user must ensure this is not called concurrently
    /// to [`Self::setup_audio_processor`] or [`Self::teardown_audio_processor`]
    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        self.audio_processor.is_some()
    }

    fn handle_panic<T, F, Pa>(param: Pa, handler: F) -> Result<T, HostWrapperError>
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

    fn ensure_initializing_called(&self) {
        // This can only happen if the plugin tried to call a host method before init().
        let Some(ptr) = self.plugin_ptr.get().copied() else {
            return;
        };

        self.init_guard.call_once_force(|_| {
            let result =
                self.init_started
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);

            // The comparison succeeded, and false was indeed the bool's value
            if result == Ok(false) {
                // SAFETY: The pointer is guaranteed to be valid by the caller of created()
                let handle =
                    unsafe { InitializingPluginHandle::new(self.destroy_lock.clone(), ptr) };
                self.shared().initializing(handle);
            }
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum HostWrapperError {
    /// An invalid parameter value was encountered.
    ///
    /// The given string may contain more information about which parameter was found to be invalid.
    InvalidParameter(&'static str),
    NullHostInstance,
    NullHostData,
    Panic,
    HostError(HostError),
}

impl HostWrapperError {
    fn msg(&self) -> &'static str {
        match self {
            HostWrapperError::NullHostInstance => "Host instance pointer is NULL",
            HostWrapperError::NullHostData => "Host data pointer is NULL",
            HostWrapperError::InvalidParameter(s) => s,
            HostWrapperError::Panic => "Host callback panicked",
            HostWrapperError::HostError(e) => e.msg(),
        }
    }

    fn severity(&self) -> clap_log_severity {
        match self {
            HostWrapperError::NullHostInstance => CLAP_LOG_PLUGIN_MISBEHAVING,
            HostWrapperError::InvalidParameter(_) => CLAP_LOG_PLUGIN_MISBEHAVING,
            HostWrapperError::NullHostData => CLAP_LOG_HOST_MISBEHAVING,
            HostWrapperError::Panic => CLAP_LOG_HOST_MISBEHAVING,
            HostWrapperError::HostError(e) => e.severity(),
        }
    }
}

impl From<HostError> for HostWrapperError {
    #[inline]
    fn from(e: HostError) -> Self {
        Self::HostError(e)
    }
}

/// # Safety
/// The user MUST ensure the Shared ref lives long enough
unsafe fn extend_shared_ref<'a, H: SharedHandler<'a>>(shared: &H) -> &'a H {
    &*(shared as *const _)
}

/// # Safety
/// The user MUST prevent this reference to be written anywhere
unsafe fn shrink_shared_ref<'a, 'instance, H: HostHandlers>(
    shared: &'a H::Shared<'instance>,
) -> &'a H::Shared<'a> {
    let original_ptr = shared as *const H::Shared<'instance>;
    let transmuted_ptr: *const H::Shared<'a> = core::mem::transmute(original_ptr);

    &*transmuted_ptr
}
