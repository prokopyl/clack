use crate::factory::{Factory, FactoryImpl};
use crate::host::HostInfo;
use crate::plugin::descriptor::RawPluginDescriptor;
use crate::plugin::PluginInstance;
use clap_sys::factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use clap_sys::host::clap_host;
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use std::ffi::CStr;

#[repr(C)]
pub struct PluginFactory<F> {
    _inner: clap_plugin_factory,
    sentinel: u8,
    factory: F,
}

impl<'a, F: PluginFactoryImpl<'a>> PluginFactory<F> {
    pub const fn new(factory: F) -> Self {
        Self {
            sentinel: 42,
            _inner: clap_plugin_factory {
                get_plugin_count: Some(Self::get_plugin_count),
                get_plugin_descriptor: Some(Self::get_plugin_descriptor),
                create_plugin: Some(Self::create_plugin),
            },
            factory,
        }
    }

    unsafe extern "C" fn get_plugin_count(factory: *const clap_plugin_factory) -> u32 {
        let this = &*(factory as *const Self);
        this.factory.plugin_count()
    }

    unsafe extern "C" fn get_plugin_descriptor(
        factory: *const clap_plugin_factory,
        index: u32,
    ) -> *const clap_plugin_descriptor {
        let this = &*(factory as *const Self);

        match this.factory.plugin_descriptor(index) {
            None => core::ptr::null(),
            Some(d) => d,
        }
    }

    unsafe extern "C" fn create_plugin(
        factory: *const clap_plugin_factory,
        clap_host: *const clap_host,
        plugin_id: *const std::os::raw::c_char,
    ) -> *const clap_plugin {
        let plugin_id = CStr::from_ptr(plugin_id);
        if clap_host.is_null() {
            eprintln!("[ERROR] Null clap_host pointer was provided to entry::create_plugin.");
            return core::ptr::null();
        };

        let host_info = HostInfo::from_raw(clap_host);
        let this = &*(factory as *const Self);

        match this.factory.create_plugin(host_info, plugin_id) {
            None => core::ptr::null(),
            Some(instance) => instance.into_owned_ptr(),
        }
    }
}

unsafe impl<F> Factory for PluginFactory<F> {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_ID;
}

unsafe impl<F> FactoryImpl for PluginFactory<F> {}

pub trait PluginFactoryImpl<'a>: 'a {
    fn plugin_count(&self) -> u32;
    fn plugin_descriptor(&self, index: u32) -> Option<&RawPluginDescriptor>;
    fn create_plugin(
        &'a self,
        host_info: HostInfo<'a>,
        plugin_id: &CStr,
    ) -> Option<PluginInstance<'a>>;
}
