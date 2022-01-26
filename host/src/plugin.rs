use clap_sys::plugin::clap_plugin;
use std::marker::PhantomData;

#[derive(Eq, PartialEq)]
pub struct PluginMainThread<'a> {
    raw: *mut clap_plugin,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> PluginMainThread<'a> {
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
    pub fn shared(&self) -> PluginShared<'a> {
        PluginShared {
            raw: self.raw,
            lifetime: PhantomData,
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct PluginShared<'a> {
    raw: *mut clap_plugin,
    lifetime: PhantomData<&'a clap_plugin>,
}

unsafe impl<'a> Send for PluginShared<'a> {}
unsafe impl<'a> Sync for PluginShared<'a> {}

impl<'a> PluginShared<'a> {
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
}

#[derive(Eq, PartialEq)]
pub struct PluginAudioProcessor<'a> {
    raw: *mut clap_plugin,
    lifetime: PhantomData<&'a clap_plugin>,
}

unsafe impl<'a> Send for PluginAudioProcessor<'a> {}

impl<'a> PluginAudioProcessor<'a> {
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
    pub fn shared(&self) -> PluginShared<'a> {
        PluginShared {
            raw: self.raw,
            lifetime: PhantomData,
        }
    }
}
