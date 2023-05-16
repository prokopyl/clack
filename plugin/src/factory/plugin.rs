use crate::factory::Factory;
use crate::factory::FactoryImplementation;
use crate::host::HostInfo;
use crate::plugin::descriptor::RawPluginDescriptor;
use crate::plugin::PluginInstance;
use clap_sys::factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use clap_sys::host::clap_host;
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use std::ffi::CStr;

#[repr(C)]
pub struct PluginFactory {
    _inner: clap_plugin_factory,
}

unsafe impl Factory for PluginFactory {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_ID;
}

pub trait PluginFactoryImpl<'a> {
    fn plugin_count() -> u32;
    fn plugin_descriptor(index: u32) -> Option<&'static RawPluginDescriptor>;
    fn create_plugin(host_info: HostInfo<'a>, plugin_id: &CStr) -> Option<PluginInstance<'a>>;
}

impl<'a, F: PluginFactoryImpl<'a>> FactoryImplementation<F> for PluginFactory {
    #[doc(hidden)]
    const IMPLEMENTATION: &'static Self = &PluginFactory {
        _inner: clap_plugin_factory {
            get_plugin_count: Some(get_plugin_count::<F>),
            get_plugin_descriptor: Some(get_plugin_descriptor::<F>),
            create_plugin: Some(create_plugin::<F>),
        },
    };
}

unsafe extern "C" fn get_plugin_count<'a, F: PluginFactoryImpl<'a>>(
    _f: *const clap_plugin_factory,
) -> u32 {
    F::plugin_count()
}

unsafe extern "C" fn get_plugin_descriptor<'a, F: PluginFactoryImpl<'a>>(
    _f: *const clap_plugin_factory,
    index: u32,
) -> *const clap_plugin_descriptor {
    match F::plugin_descriptor(index) {
        None => core::ptr::null(),
        Some(d) => d,
    }
}

unsafe extern "C" fn create_plugin<'a, F: PluginFactoryImpl<'a>>(
    _f: *const clap_plugin_factory,
    clap_host: *const clap_host,
    plugin_id: *const std::os::raw::c_char,
) -> *const clap_plugin {
    let plugin_id = CStr::from_ptr(plugin_id);
    if clap_host.is_null() {
        eprintln!("[ERROR] Null clap_host pointer was provided to entry::create_plugin.");
        return core::ptr::null();
    };

    let host_info = HostInfo::from_raw(clap_host);

    match F::create_plugin(host_info, plugin_id) {
        None => core::ptr::null(),
        Some(instance) => instance.into_owned_ptr(),
    }
}
