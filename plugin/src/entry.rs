use crate::host::HostInfo;
use crate::plugin::{Plugin, PluginDescriptor, PluginInstance};
pub use clack_common::entry::PluginEntryDescriptor;
use clap_sys::{
    host::clap_host,
    plugin::{
        clap_plugin, clap_plugin_descriptor, clap_plugin_entry, clap_plugin_invalidation_source,
    },
    version::CLAP_VERSION,
};
use std::ffi::CStr;
use std::marker::PhantomData;

pub trait PluginEntry: Sized {
    fn init(_plugin_path: &CStr) {}
    fn de_init() {}

    fn plugin_count() -> u32;
    fn plugin_descriptor(index: u32) -> Option<&'static PluginDescriptor>;
    fn create_plugin<'a>(host_info: HostInfo<'a>, plugin_id: &[u8]) -> Option<PluginInstance<'a>>;

    const DESCRIPTOR: PluginEntryDescriptor = PluginEntryDescriptor::new(clap_plugin_entry {
        clap_version: CLAP_VERSION,
        init: init::<Self>,
        deinit: de_init::<Self>,
        get_plugin_count: get_plugin_count::<Self>,
        get_plugin_descriptor: get_plugin_descriptor::<Self>,
        create_plugin: create_plugin::<Self>,
        get_invalidation_source_count: get_invalidation_source_count::<Self>,
        get_invalidation_source: get_invalidation_source::<Self>,
        refresh: refresh::<Self>,
    });
}

unsafe extern "C" fn init<E: PluginEntry>(plugin_path: *const ::std::os::raw::c_char) {
    E::init(CStr::from_ptr(plugin_path));
}

unsafe extern "C" fn de_init<E: PluginEntry>() {
    E::de_init()
}

unsafe extern "C" fn get_plugin_count<E: PluginEntry>() -> u32 {
    E::plugin_count()
}

unsafe extern "C" fn get_plugin_descriptor<E: PluginEntry>(
    index: u32,
) -> *const clap_plugin_descriptor {
    match E::plugin_descriptor(index) {
        None => ::core::ptr::null(),
        Some(d) => &d.0,
    }
}

unsafe extern "C" fn create_plugin<E: PluginEntry>(
    clap_host: *const clap_host,
    plugin_id: *const std::os::raw::c_char,
) -> *const clap_plugin {
    let plugin_id = CStr::from_ptr(plugin_id).to_bytes_with_nul();
    let clap_host = if let Some(clap_host) = clap_host.as_ref() {
        clap_host
    } else {
        eprintln!("[ERROR] Null clap_host pointer was provided to entry::create_plugin.");
        return ::core::ptr::null();
    };

    let host_info = HostInfo { inner: clap_host };

    match E::create_plugin(host_info, plugin_id) {
        None => ::core::ptr::null(),
        Some(instance) => instance.into_owned_ptr(),
    }
}

unsafe extern "C" fn get_invalidation_source_count<E: PluginEntry>() -> u32 {
    0 // TODO
}

unsafe extern "C" fn get_invalidation_source<E: PluginEntry>(
    _index: u32,
) -> *const clap_plugin_invalidation_source {
    ::core::ptr::null() // TODO
}

unsafe extern "C" fn refresh<E: PluginEntry>() {
    // TODO
}

pub struct SinglePluginEntry<P: for<'a> Plugin<'a>>(PhantomData<P>);

impl<P: for<'a> Plugin<'a>> PluginEntry for SinglePluginEntry<P> {
    #[inline]
    fn plugin_count() -> u32 {
        1
    }

    #[inline]
    fn plugin_descriptor(index: u32) -> Option<&'static PluginDescriptor> {
        match index {
            0 => Some(P::DESCRIPTOR),
            _ => None,
        }
    }

    #[inline]
    fn create_plugin<'p>(host_info: HostInfo<'p>, plugin_id: &[u8]) -> Option<PluginInstance<'p>> {
        if plugin_id == P::ID {
            Some(PluginInstance::<'p>::new::<P>(host_info))
        } else {
            None
        }
    }
}
