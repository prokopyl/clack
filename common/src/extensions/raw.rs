use crate::extensions::{ExtensionSide, HostExtensionSide, PluginExtensionSide};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Copy, Clone)]
pub struct RawExtension<S: ExtensionSide, T = ()> {
    extension_ptr: NonNull<T>,
    host_or_plugin_ptr: NonNull<c_void>, // Can be either clap_host or clap_plugin
    _side: PhantomData<fn() -> S>,
}

impl<S: ExtensionSide, T> PartialEq for RawExtension<S, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.extension_ptr == other.extension_ptr
    }
}

impl<S: ExtensionSide, T> Eq for RawExtension<S, T> {}

// SAFETY: this is just a couple of pointers, and the type doesn't care being used on any thread.
// Thread-safety is enforced by the plugin handle type that is passed to the methods.
unsafe impl<S: ExtensionSide, T> Send for RawExtension<S, T> {}
// SAFETY: same as above.
unsafe impl<S: ExtensionSide, T> Sync for RawExtension<S, T> {}

impl<S: ExtensionSide, T> RawExtension<S, T> {
    #[inline]
    pub fn as_ptr(&self) -> NonNull<T> {
        self.extension_ptr
    }
}

impl<S: ExtensionSide> RawExtension<S, ()> {
    /// Casts this raw extension pointer into an extension pointer of a given type.
    ///
    /// # Safety
    ///
    /// Users *must* ensure that `T` matches the actual type behind the pointer.
    pub unsafe fn cast<T>(&self) -> RawExtension<S, T> {
        RawExtension {
            extension_ptr: self.extension_ptr.cast(),
            host_or_plugin_ptr: self.host_or_plugin_ptr,
            _side: PhantomData,
        }
    }
}

impl<T> RawExtension<PluginExtensionSide, T> {
    /// Creates a raw plugin-side extension pointer from a pointer to the extension data, and a
    /// pointer to the plugin instance.
    ///
    /// # Safety
    ///
    /// The user *must* ensure the `extension_ptr` is and remains valid for the lifetime of the
    /// plugin instance.
    ///
    /// The given `plugin_ptr` however doesn't have to be valid, and may be dangling.
    pub unsafe fn from_raw(extension_ptr: NonNull<T>, plugin_ptr: NonNull<clap_plugin>) -> Self {
        Self {
            extension_ptr,
            host_or_plugin_ptr: plugin_ptr.cast(),
            _side: PhantomData,
        }
    }

    // TODO: docs: this can be dangling
    pub fn plugin_ptr(&self) -> NonNull<clap_plugin> {
        self.host_or_plugin_ptr.cast()
    }
}

impl<T> RawExtension<HostExtensionSide, T> {
    /// Creates a raw host-side extension pointer from a pointer to the extension data, and a
    /// pointer to the plugin instance.
    ///
    /// # Safety
    ///
    /// The user *must* ensure the `extension_ptr` is and remains valid for the lifetime of the
    /// plugin instance.
    ///
    /// The given `host_ptr` however doesn't have to be valid, and may be dangling.
    pub unsafe fn from_raw(extension_ptr: NonNull<T>, host_ptr: NonNull<clap_host>) -> Self {
        Self {
            extension_ptr,
            host_or_plugin_ptr: host_ptr.cast(),
            _side: PhantomData,
        }
    }

    // TODO: docs: this can be dangling
    pub fn host_ptr(&self) -> NonNull<clap_host> {
        self.host_or_plugin_ptr.cast()
    }
}

pub struct RawExtensionImplementation {
    inner: NonNull<c_void>,
}

impl RawExtensionImplementation {
    pub const fn new<I>(implementation: &'static I) -> Self {
        Self {
            // SAFETY: pointer comes from a reference, so it's guaranteed to always be valid.
            inner: unsafe { NonNull::new_unchecked(implementation as *const _ as *mut _) },
        }
    }

    pub const fn as_ptr(&self) -> NonNull<c_void> {
        self.inner
    }
}
