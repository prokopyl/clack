use clack_common::extensions::{Extension, PluginExtensionSide};
use clap_sys::plugin::clap_plugin;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::NonNull;

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
        debug_handle("PluginMainThreadHandle", self.raw, f)
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
        debug_handle("PluginSharedHandle", self.raw, f)
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
        debug_handle("PluginAudioProcessorHandle", self.raw, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PluginInitializingHandle<'a> {
    raw: NonNull<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin>,
}

// SAFETY: The Shared handle only exposes thread-safe methods
unsafe impl<'a> Send for PluginInitializingHandle<'a> {}
// SAFETY: The Shared handle only exposes thread-safe methods
unsafe impl<'a> Sync for PluginInitializingHandle<'a> {}

impl<'a> PluginInitializingHandle<'a> {
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

impl Debug for PluginInitializingHandle<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        debug_handle("PluginInitializingHandle", self.raw, f)
    }
}

fn debug_handle(name: &str, ptr: NonNull<clap_plugin>, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "{name} ({ptr:p})")
}
