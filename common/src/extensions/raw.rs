use crate::extensions::{ExtensionSide, HostExtensionSide, PluginExtensionSide};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Copy, Clone)]
pub struct RawExtension<S: ExtensionSide, T = c_void> {
    extension_ptr: NonNull<T>,
    host_or_plugin_ptr: NonNull<c_void>, // Can be either clap_host or clap_plugin
    _side: PhantomData<fn() -> S>,
}

// TODO: impl Eq + PartialEq

// SAFETY: this is just a couple of pointers, and the type doesn't care being used on any thread.
// Thread-safety is enforced by the plugin handle type that is passed to the methods.
unsafe impl<S: ExtensionSide, T> Send for RawExtension<S, T> {}
// SAFETY: same as above.
unsafe impl<S: ExtensionSide, T> Sync for RawExtension<S, T> {}

impl<S: ExtensionSide, T> RawExtension<S, T> {
    pub unsafe fn cast<U>(&self) -> RawExtension<S, U> {
        RawExtension {
            extension_ptr: self.extension_ptr.cast(),
            host_or_plugin_ptr: self.host_or_plugin_ptr,
            _side: PhantomData,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> NonNull<T> {
        self.extension_ptr
    }
}

impl<T> RawExtension<PluginExtensionSide, T> {
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
