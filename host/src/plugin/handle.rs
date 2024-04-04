use clack_common::extensions::{Extension, PluginExtensionSide};
use clap_sys::plugin::clap_plugin;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Eq, PartialEq)]
pub struct PluginMainThreadHandle<'a> {
    raw: NonNull<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> PluginMainThreadHandle<'a> {
    /// # Safety
    /// The user must ensure the provided plugin pointer is valid.
    /// This can only be called on the main thread.
    pub(crate) unsafe fn new(raw: NonNull<clap_plugin>) -> Self {
        Self {
            raw,
            lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> *mut clap_plugin {
        self.raw.as_ptr()
    }

    #[inline]
    pub fn shared(&self) -> PluginSharedHandle<'a> {
        // SAFETY: This type ensures the provided pointer is valid for 'a
        unsafe { PluginSharedHandle::new(self.raw) }
    }
}

impl Debug for PluginMainThreadHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluginMainThreadHandle ({:p})", self.raw)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PluginSharedHandle<'a> {
    raw: NonNull<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin>,
}

// SAFETY: The Shared handle only exposes thread-safe methods
unsafe impl<'a> Send for PluginSharedHandle<'a> {}
// SAFETY: The Shared handle only exposes thread-safe methods
unsafe impl<'a> Sync for PluginSharedHandle<'a> {}

impl<'a> PluginSharedHandle<'a> {
    /// # Safety
    ///
    /// Users must ensure the provided instance pointer is valid.
    pub(crate) unsafe fn new(raw: NonNull<clap_plugin>) -> Self {
        Self {
            raw,
            lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> *const clap_plugin {
        self.raw.as_ptr()
    }

    pub fn get_extension<E: Extension<ExtensionSide = PluginExtensionSide>>(
        &self,
    ) -> Option<&'a E> {
        // SAFETY: This type ensures the function pointer is valid
        let ext =
            unsafe { self.raw.as_ref().get_extension?(self.raw.as_ptr(), E::IDENTIFIER.as_ptr()) };
        // SAFETY: Extension is valid for the instance's lifetime 'a, and pointer comes from E's Identifier
        NonNull::new(ext as *mut _).map(|p| unsafe { E::from_extension_ptr(p) })
    }
}
impl Debug for PluginSharedHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluginSharedHandle ({:p})", self.raw)
    }
}

#[derive(Eq, PartialEq)]
pub struct PluginAudioProcessorHandle<'a> {
    raw: NonNull<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin>,
}

// SAFETY: This type only exposes audio-thread methods
unsafe impl<'a> Send for PluginAudioProcessorHandle<'a> {}

impl<'a> PluginAudioProcessorHandle<'a> {
    pub(crate) fn new(raw: NonNull<clap_plugin>) -> Self {
        Self {
            raw,
            lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> *mut clap_plugin {
        self.raw.as_ptr()
    }

    #[inline]
    pub fn shared(&self) -> PluginSharedHandle<'a> {
        PluginSharedHandle {
            raw: self.raw,
            lifetime: PhantomData,
        }
    }
}
impl Debug for PluginAudioProcessorHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluginAudioProcessorHandle ({:p})", self.raw)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct InitializingPluginHandle<'a> {
    inner: RemoteHandleInner,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> InitializingPluginHandle<'a> {
    /// # Safety
    ///
    /// Users must ensure the provided instance pointer is valid until the given lock is marked as destroying.
    pub(crate) unsafe fn new(lock: Arc<DestroyLock>, instance: NonNull<clap_plugin>) -> Self {
        Self {
            lifetime: PhantomData,
            inner: RemoteHandleInner { instance, lock },
        }
    }

    #[inline]
    pub fn as_raw(&self) -> NonNull<clap_plugin> {
        self.inner.as_ptr()
    }

    pub fn get_extension<E: Extension<ExtensionSide = PluginExtensionSide>>(
        &self,
    ) -> Option<&'a E> {
        self.inner.get_extension()
    }
}

impl Debug for InitializingPluginHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InitializingPluginHandle ({:p})", self.inner.instance)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct InitializedPluginHandle<'a> {
    inner: RemoteHandleInner,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> InitializedPluginHandle<'a> {
    /// # Safety
    ///
    /// Users must ensure the provided instance pointer is valid until the given lock is marked as destroying.
    pub(crate) unsafe fn new(lock: Arc<DestroyLock>, instance: NonNull<clap_plugin>) -> Self {
        Self {
            lifetime: PhantomData,
            inner: RemoteHandleInner { instance, lock },
        }
    }

    #[inline]
    pub fn as_raw(&self) -> NonNull<clap_plugin> {
        self.inner.as_ptr()
    }

    // TODO: bikeshed?
    #[inline]
    pub fn access<T>(&self, handler: impl FnOnce(PluginSharedHandle) -> T) -> Option<T> {
        self.inner.handle(handler)
    }

    // FIXME: bogus extension lifetime
    pub fn get_extension<E: Extension<ExtensionSide = PluginExtensionSide>>(
        &self,
    ) -> Option<&'a E> {
        self.inner.get_extension()
    }
}

impl Debug for InitializedPluginHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InitializingPluginHandle ({:p})", self.inner.instance)
    }
}

#[derive(Clone)]
struct RemoteHandleInner {
    lock: Arc<DestroyLock>,
    instance: NonNull<clap_plugin>,
}

impl RemoteHandleInner {
    #[inline]
    fn as_ptr(&self) -> NonNull<clap_plugin> {
        self.instance
    }

    fn handle<T>(&self, handler: impl FnOnce(PluginSharedHandle) -> T) -> Option<T> {
        self.lock.hold_off_destruction(|| {
            // SAFETY: this type ensures the plugin is not being destroyed yet.
            let handle = unsafe { PluginSharedHandle::new(self.instance) };
            handler(handle)
        })
    }

    // FIXME: extension pointers may become invalid after plugin destruction, so the lifetime here is bogus
    fn get_extension<'a, E: Extension<ExtensionSide = PluginExtensionSide>>(
        &self,
    ) -> Option<&'a E> {
        self.handle(|handle| {
            // SAFETY: This type ensures the function pointer is valid
            let ext = unsafe {
                handle.raw.as_ref().get_extension?(handle.raw.as_ptr(), E::IDENTIFIER.as_ptr())
            };
            // SAFETY: Extension is valid for the instance's lifetime 'a, and pointer comes from E's Identifier
            NonNull::new(ext as *mut _).map(|p| unsafe { E::from_extension_ptr(p) })
        })?
    }
}

impl PartialEq for RemoteHandleInner {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.instance == other.instance
    }
}

impl Eq for RemoteHandleInner {}

// SAFETY: The Shared handles only exposes thread-safe methods
unsafe impl Send for RemoteHandleInner {}
// SAFETY: The Shared handles only exposes thread-safe methods
unsafe impl Sync for RemoteHandleInner {}

pub(crate) struct DestroyLock {
    is_destroying: AtomicBool,
    lock: RwLock<bool>,
}

impl DestroyLock {
    pub(crate) fn new() -> Self {
        Self {
            is_destroying: AtomicBool::new(false),
            lock: RwLock::new(false),
        }
    }

    pub(crate) fn start_destroying(&self) {
        // Notify threads that may use the lock in the future that we are about to start destroying.
        self.is_destroying.store(true, Ordering::SeqCst);

        self.lock.clear_poison();
        let mut guard = self.lock.write().unwrap_or_else(|err| err.into_inner());
        // This additional check may not be very useful, it's there just in case.
        *guard = true;
    }

    fn hold_off_destruction<T>(&self, handler: impl FnOnce() -> T) -> Option<T> {
        if self.is_destroying.load(Ordering::SeqCst) {
            return None;
        }

        // Poisoning doesn't matter, we are only reading
        let guard = self.lock.read().unwrap_or_else(|err| err.into_inner());
        if *guard {
            return None;
        }

        let result = handler();

        drop(guard);

        Some(result)
    }
}
