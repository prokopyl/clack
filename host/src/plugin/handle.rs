use crate::factory::PluginDescriptor;
use clack_common::extensions::{Extension, PluginExtensionSide, RawExtension};
use clap_sys::plugin::clap_plugin;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Eq, PartialEq)]
#[repr(transparent)]
pub struct PluginMainThreadHandle<'a> {
    raw: NonNull<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> PluginMainThreadHandle<'a> {
    /// # Safety
    /// The user must ensure the provided plugin pointer is valid.
    /// This can only be called on the main thread.
    pub(crate) const unsafe fn new(raw: NonNull<clap_plugin>) -> Self {
        Self {
            raw,
            lifetime: PhantomData,
        }
    }

    /// Returns a shared reference to the raw, C-FFI compatible plugin instance struct.
    ///
    /// This type enforces that the reference is valid for the lifetime of the instance (`'a`).
    ///
    /// If you need to access the raw pointer without dereferencing it first, use
    /// [`as_raw_ptr`](Self::as_raw_ptr) instead.
    #[inline]
    pub const fn as_raw(&self) -> &'a clap_plugin {
        // SAFETY: this type enforces that the clap_plugin instance is valid for 'a.
        unsafe { self.raw.as_ref() }
    }

    /// Returns a raw pointer to the raw, C-FFI compatible plugin instance struct, without dereferencing it.
    ///
    /// If you need to safely access the plugin instance struct through a shared reference,
    /// use [`as_raw`](Self::as_raw) instead.
    #[inline]
    pub const fn as_raw_ptr(&self) -> *const clap_plugin {
        self.raw.as_ptr()
    }

    #[inline]
    pub const fn shared(&self) -> PluginSharedHandle<'a> {
        // SAFETY: This type ensures the provided pointer is valid for 'a
        unsafe { PluginSharedHandle::new(self.raw) }
    }

    #[inline]
    pub const fn as_shared(&self) -> &PluginSharedHandle<'a> {
        // SAFETY: this cast is valid since both types are just a NonNull<clap_host> and repr(transparent)
        unsafe { &*(self as *const Self as *const PluginSharedHandle<'a>) }
    }
}

impl Debug for PluginMainThreadHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluginMainThreadHandle ({:p})", self.raw)
    }
}

impl<'a> Deref for PluginMainThreadHandle<'a> {
    type Target = PluginSharedHandle<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_shared()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct PluginSharedHandle<'a> {
    raw: NonNull<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin>,
}

// SAFETY: The Shared handle only exposes thread-safe methods
unsafe impl Send for PluginSharedHandle<'_> {}
// SAFETY: The Shared handle only exposes thread-safe methods
unsafe impl Sync for PluginSharedHandle<'_> {}

impl<'a> PluginSharedHandle<'a> {
    /// # Safety
    ///
    /// Users must ensure the provided instance pointer is valid.
    pub(crate) const unsafe fn new(raw: NonNull<clap_plugin>) -> Self {
        Self {
            raw,
            lifetime: PhantomData,
        }
    }

    /// Returns the [`PluginDescriptor`] this instance corresponds to.
    ///
    /// This may return `None` if the underlying plugin implementation didn't properly populate
    /// the descriptor pointer.
    pub const fn descriptor(&self) -> Option<PluginDescriptor<'a>> {
        // SAFETY: the desc pointer is guaranteed to be valid (if present) by the CLAP spec.
        let Some(descriptor) = (unsafe { self.as_raw().desc.as_ref() }) else {
            return None;
        };

        // SAFETY: the desc pointer is guaranteed to be valid (if present) by the CLAP spec.
        Some(unsafe { PluginDescriptor::from_raw(descriptor) })
    }

    /// Returns a shared reference to the raw, C-FFI compatible plugin instance struct.
    ///
    /// This type enforces that the reference is valid for the lifetime of the instance (`'a`).
    ///
    /// If you need to access the raw pointer without dereferencing it first, use
    /// [`as_raw_ptr`](Self::as_raw_ptr) instead.
    #[inline]
    pub const fn as_raw(&self) -> &'a clap_plugin {
        // SAFETY: this type enforces that the clap_plugin instance is valid for 'a.
        unsafe { self.raw.as_ref() }
    }

    /// Returns a raw pointer to the raw, C-FFI compatible plugin instance struct, without dereferencing it.
    ///
    /// If you need to safely access the plugin instance struct through a shared reference,
    /// use [`as_raw`](Self::as_raw) instead.
    #[inline]
    pub const fn as_raw_ptr(&self) -> *const clap_plugin {
        self.raw.as_ptr()
    }

    pub fn get_extension<E: Extension<ExtensionSide = PluginExtensionSide>>(&self) -> Option<E> {
        let identifier = const { E::IDENTIFIERS.first().unwrap() };
        // SAFETY: This type ensures the function pointers are valid
        let ext = unsafe { self.as_raw().get_extension?(self.raw.as_ptr(), identifier.as_ptr()) };

        let ext = NonNull::new(ext as *mut _)?;
        // SAFETY: The CLAP spec guarantees that the extension lives as long as the instance.
        let raw = unsafe { RawExtension::from_raw_plugin_extension(ext, self.raw) };

        // SAFETY: pointer comes from the associated E::IDENTIFIER.
        unsafe { Some(E::from_raw(raw)) }
    }

    /// Safely dereferences a [`RawExtension`] pointer produced by this plugin instance.
    ///
    /// See the documentation of the [`RawExtension`] type for more information about how this works
    /// internally.
    ///
    /// # Panics
    ///
    /// This method will panic if the given extension pointer does not match the plugin instance of
    /// this handle.
    #[inline]
    pub fn use_extension<E: Sized>(
        &self,
        extension: &RawExtension<PluginExtensionSide, E>,
    ) -> &'a E {
        if self.raw != extension.plugin_ptr() {
            mismatched_instance();
        }

        // SAFETY: the RawExtension type enforces the pointee is valid for as long as the matching
        // instance is still alive.
        unsafe { extension.as_ptr().as_ref() }
    }
}

impl Debug for PluginSharedHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluginSharedHandle ({:p})", self.raw)
    }
}

#[derive(Eq, PartialEq)]
#[repr(transparent)]
pub struct PluginAudioProcessorHandle<'a> {
    raw: NonNull<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin>,
}

// SAFETY: This type only exposes audio-thread methods
unsafe impl Send for PluginAudioProcessorHandle<'_> {}

impl<'a> PluginAudioProcessorHandle<'a> {
    pub(crate) const fn new(raw: NonNull<clap_plugin>) -> Self {
        Self {
            raw,
            lifetime: PhantomData,
        }
    }

    /// Returns a shared reference to the raw, C-FFI compatible plugin instance struct.
    ///
    /// This type enforces that the reference is valid for the lifetime of the instance (`'a`).
    ///
    /// If you need to access the raw pointer without dereferencing it first, use
    /// [`as_raw_ptr`](Self::as_raw_ptr) instead.
    #[inline]
    pub const fn as_raw(&self) -> &'a clap_plugin {
        // SAFETY: this type enforces that the clap_plugin instance is valid for 'a.
        unsafe { self.raw.as_ref() }
    }

    /// Returns a raw pointer to the raw, C-FFI compatible plugin instance struct, without dereferencing it.
    ///
    /// If you need to safely access the plugin instance struct through a shared reference,
    /// use [`as_raw`](Self::as_raw) instead.
    #[inline]
    pub const fn as_raw_ptr(&self) -> *const clap_plugin {
        self.raw.as_ptr()
    }

    #[inline]
    pub const fn shared(&self) -> PluginSharedHandle<'a> {
        PluginSharedHandle {
            raw: self.raw,
            lifetime: PhantomData,
        }
    }

    #[inline]
    pub const fn as_shared(&self) -> &PluginSharedHandle<'a> {
        // SAFETY: this cast is valid since both types are just a NonNull<clap_host> and repr(transparent)
        unsafe { &*(self as *const Self as *const PluginSharedHandle<'a>) }
    }
}

impl<'a> Deref for PluginAudioProcessorHandle<'a> {
    type Target = PluginSharedHandle<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_shared()
    }
}

impl Debug for PluginAudioProcessorHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluginAudioProcessorHandle ({:p})", self.raw)
    }
}

/// A handle to a plugin instance that may be in the process of initializing.
///
/// In this state, only [querying plugin extensions](Self::get_extension) is allowed.
#[derive(Clone, Eq, PartialEq)]
pub struct InitializingPluginHandle<'a> {
    inner: RemoteHandleInner,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> InitializingPluginHandle<'a> {
    /// # Safety
    ///
    /// Users must ensure the provided instance pointer is valid until the given lock is marked as destroying.
    pub(crate) const unsafe fn new(lock: Arc<DestroyLock>, instance: NonNull<clap_plugin>) -> Self {
        Self {
            lifetime: PhantomData,
            inner: RemoteHandleInner { instance, lock },
        }
    }

    /// Returns a shared reference to the raw, C-FFI compatible plugin instance struct.
    ///
    /// This type enforces that the reference is valid for the lifetime of the instance (`'a`).
    ///
    /// If you need to access the raw pointer without dereferencing it first, use
    /// [`as_raw_ptr`](Self::as_raw_ptr) instead.
    #[inline]
    pub const fn as_raw(&self) -> &'a clap_plugin {
        // SAFETY: this type enforces that the clap_plugin instance is valid for 'a.
        unsafe { self.inner.instance.as_ref() }
    }

    /// Returns a raw pointer to the raw, C-FFI compatible plugin instance struct, without dereferencing it.
    ///
    /// If you need to safely access the plugin instance struct through a shared reference,
    /// use [`as_raw`](Self::as_raw) instead.
    #[inline]
    pub const fn as_raw_ptr(&self) -> *const clap_plugin {
        self.inner.instance.as_ptr()
    }

    pub fn get_extension<E: Extension<ExtensionSide = PluginExtensionSide>>(&self) -> Option<E> {
        self.inner.get_extension()
    }
}

impl Debug for InitializingPluginHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InitializingPluginHandle ({:p})", self.inner.instance)
    }
}

/// A handle to a plugin instance that has finished initializing.
///
/// This handle can be used to obtain a [`PluginSharedHandle`] to then call the plugin's thread-safe
/// method.
///
/// However, this handle can outlive the plugin instance, as host callbacks may be called during
/// the plugin's destruction.
///
/// Therefore, the [`PluginSharedHandle`] can only be accessed through the [`access`](Self::access)
/// method, ensuring no access can be made during or after destruction.
#[derive(Clone, Eq, PartialEq)]
pub struct InitializedPluginHandle<'a> {
    inner: RemoteHandleInner,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> InitializedPluginHandle<'a> {
    /// # Safety
    ///
    /// Users must ensure the provided instance pointer is valid until the given lock is marked as destroying.
    pub(crate) const unsafe fn new(lock: Arc<DestroyLock>, instance: NonNull<clap_plugin>) -> Self {
        Self {
            lifetime: PhantomData,
            inner: RemoteHandleInner { instance, lock },
        }
    }

    /// Returns a shared reference to the raw, C-FFI compatible plugin instance struct.
    ///
    /// This type enforces that the reference is valid for the lifetime of the instance (`'a`).
    ///
    /// If you need to access the raw pointer without dereferencing it first, use
    /// [`as_raw_ptr`](Self::as_raw_ptr) instead.
    #[inline]
    pub const fn as_raw(&self) -> &'a clap_plugin {
        // SAFETY: this type enforces that the clap_plugin instance is valid for 'a.
        unsafe { self.inner.instance.as_ref() }
    }

    /// Returns a raw pointer to the raw, C-FFI compatible plugin instance struct, without dereferencing it.
    ///
    /// If you need to safely access the plugin instance struct through a shared reference,
    /// use [`as_raw`](Self::as_raw) instead.
    #[inline]
    pub const fn as_raw_ptr(&self) -> *const clap_plugin {
        self.inner.instance.as_ptr()
    }

    /// Attempts to access and perform the given operation on the plugin's handle, returning its
    /// result.
    ///
    /// This method will return `None` if the plugin instance is being or has been destroyed, and
    /// `f` will not be called.
    ///
    /// However, this method guarantees that the plugin instance will block starting its destruction
    /// while the given operation `f` is still ongoing.
    ///
    /// This ensures the plugin instance is always valid if and when `f` runs.
    ///
    /// # Realtime safety
    ///
    /// This method *may* block the current thread if it is called concurrently while the plugin's
    /// instance is being destroyed (i.e. [`PluginInstance::drop`] is running).
    ///
    /// [`PluginInstance::drop`]: crate::plugin::PluginInstance::drop
    #[inline]
    pub fn access<T>(&self, f: impl FnOnce(PluginSharedHandle) -> T) -> Option<T> {
        self.inner.access(f)
    }

    pub fn get_extension<E: Extension<ExtensionSide = PluginExtensionSide>>(&self) -> Option<E> {
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
    fn access<T>(&self, f: impl FnOnce(PluginSharedHandle) -> T) -> Option<T> {
        self.lock.hold_off_destruction(|| {
            // SAFETY: this type ensures the plugin is not being destroyed yet.
            let handle = unsafe { PluginSharedHandle::new(self.instance) };
            f(handle)
        })
    }

    #[inline]
    fn get_extension<E: Extension<ExtensionSide = PluginExtensionSide>>(&self) -> Option<E> {
        self.access(|handle| handle.get_extension())?
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

        let mut guard = self.lock.write().unwrap_or_else(|err| err.into_inner());
        // This additional check may not be very useful, it's there just in case.
        *guard = true;
    }

    fn hold_off_destruction<T>(&self, handler: impl FnOnce() -> T) -> Option<T> {
        if self.is_destroying.load(Ordering::SeqCst) {
            return None;
        }

        // Poisoning doesn't matter, we are only reading a bool
        let guard = self.lock.read().unwrap_or_else(|err| err.into_inner());
        if *guard {
            return None;
        }

        let result = handler();

        drop(guard);

        Some(result)
    }
}

#[cold]
const fn mismatched_instance() -> ! {
    panic!("Given plugin instance handle doesn't match the extension pointer it was used on.")
}
