use crate::factory::{Factory, RawFactoryPointer};
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use std::ffi::CStr;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginFactory<'a>(RawFactoryPointer<'a, clap_plugin_factory>);

// SAFETY: TODO
unsafe impl<'a> Factory<'a> for PluginFactory<'a> {
    const IDENTIFIERS: &'static [&'static CStr] = &[CLAP_PLUGIN_FACTORY_ID];
    type Raw = clap_plugin_factory;

    #[inline]
    fn from_raw(raw: RawFactoryPointer<'a, Self::Raw>) -> Self {
        Self(raw)
    }
}
