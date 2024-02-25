use clack_common::extensions::{Extension, PluginExtensionSide};
use clap_sys::plugin::clap_plugin;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Eq, PartialEq)]
pub struct PluginMainThreadHandle<'a> {
    raw: *mut clap_plugin,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> PluginMainThreadHandle<'a> {
    /// # Safety
    /// The user must ensure the provided plugin pointer is valid.
    /// This can only be called on the main thread.
    pub(crate) unsafe fn new(raw: *mut clap_plugin) -> Self {
        Self {
            raw,
            lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> *mut clap_plugin {
        self.raw
    }

    #[inline]
    pub fn shared(&self) -> PluginSharedHandle<'a> {
        PluginSharedHandle {
            raw: self.raw,
            lifetime: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PluginSharedHandle<'a> {
    raw: *const clap_plugin,
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
    pub(crate) unsafe fn new(raw: *const clap_plugin) -> Self {
        Self {
            raw,
            lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> *const clap_plugin {
        self.raw
    }

    pub fn get_extension<E: Extension<ExtensionSide = PluginExtensionSide>>(
        &self,
    ) -> Option<&'a E> {
        // SAFETY: This type ensures the function pointer is valid
        let ext = unsafe { (*self.raw).get_extension?(self.raw, E::IDENTIFIER.as_ptr()) } as *mut _;
        // SAFETY: Extension is valid for the instance's lifetime 'a, and pointer comes from E's Identifier
        NonNull::new(ext).map(|p| unsafe { E::from_extension_ptr(p) })
    }
}

#[derive(Eq, PartialEq)]
pub struct PluginAudioProcessorHandle<'a> {
    raw: *mut clap_plugin,
    lifetime: PhantomData<&'a clap_plugin>,
}

// SAFETY: This type only exposes audio-thread methods
unsafe impl<'a> Send for PluginAudioProcessorHandle<'a> {}

impl<'a> PluginAudioProcessorHandle<'a> {
    pub(crate) fn new(raw: *mut clap_plugin) -> Self {
        Self {
            raw,
            lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> *mut clap_plugin {
        self.raw
    }

    #[inline]
    pub fn shared(&self) -> PluginSharedHandle<'a> {
        PluginSharedHandle {
            raw: self.raw,
            lifetime: PhantomData,
        }
    }
}
