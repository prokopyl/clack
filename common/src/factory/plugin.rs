use crate::factory::Factory;
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use std::ffi::CStr;

pub struct PluginFactory {
    pub inner: clap_plugin_factory, // TODO: should not be pub
}

unsafe impl Factory for PluginFactory {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_ID;
}
