use crate::extensions::{ExtensionSide, HostExtensionSide, PluginExtensionSide};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Copy, Clone)]
pub struct RawExtension<S: ExtensionSide, T = c_void> {
    extension_ptr: NonNull<T>,
    host_or_plugin_ptr: Option<NonNull<c_void>>, // Can be either clap_host or clap_plugin
    _side: PhantomData<fn() -> S>,
}

// TODO: impl Eq + PartialEq

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
            host_or_plugin_ptr: Some(plugin_ptr.cast()),
            _side: PhantomData,
        }
    }

    // TODO: docs: this can be dangling
    pub fn plugin_ptr(&self) -> NonNull<clap_plugin> {
        // TODO: check / explain unwrap?
        self.host_or_plugin_ptr.unwrap().cast()
    }
}

impl<T> RawExtension<HostExtensionSide, T> {
    pub unsafe fn from_raw(extension_ptr: NonNull<T>, host_ptr: NonNull<clap_host>) -> Self {
        Self {
            extension_ptr,
            host_or_plugin_ptr: Some(host_ptr.cast()),
            _side: PhantomData,
        }
    }

    // TODO: docs: this can be dangling
    pub fn host_ptr(&self) -> NonNull<clap_host> {
        // TODO: check / explain unwrap?
        self.host_or_plugin_ptr.unwrap().cast()
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
