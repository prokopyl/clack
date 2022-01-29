use crate::factory::Factory;
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};

pub struct PluginFactory {
    pub inner: clap_plugin_factory, // TODO: should not be pub
}

unsafe impl<'a> Factory<'a> for PluginFactory {
    const IDENTIFIER: &'static [u8] = CLAP_PLUGIN_FACTORY_ID;
}
