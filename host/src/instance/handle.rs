use clack_common::extensions::{Extension, PluginExtensionType};
use clap_sys::plugin::clap_plugin;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Eq, PartialEq)]
pub struct PluginMainThreadHandle<'a> {
    raw: *mut clap_plugin,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> PluginMainThreadHandle<'a> {
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PluginSharedHandle<'a> {
    raw: *mut clap_plugin,
    lifetime: PhantomData<&'a clap_plugin>,
}

unsafe impl<'a> Send for PluginSharedHandle<'a> {}
unsafe impl<'a> Sync for PluginSharedHandle<'a> {}

impl<'a> PluginSharedHandle<'a> {
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

    pub fn get_extension<E: Extension<ExtensionType = PluginExtensionType>>(
        &self,
    ) -> Option<&'a E> {
        let ext =
            unsafe { ((*self.raw).get_extension?)(self.raw, E::IDENTIFIER.as_ptr()) } as *mut _;
        NonNull::new(ext).map(|p| unsafe { E::from_extension_ptr(p) })
    }
}

#[derive(Eq, PartialEq)]
pub struct PluginAudioProcessorHandle<'a> {
    raw: *mut clap_plugin,
    lifetime: PhantomData<&'a clap_plugin>,
}

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
