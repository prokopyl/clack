use crate::plugin::{PluginDescriptor, PluginInstance};
use clap_audio_common::host::HostInfo;
use clap_sys::{
    host::clap_host,
    plugin::{
        clap_plugin, clap_plugin_descriptor, clap_plugin_entry, clap_plugin_invalidation_source,
    },
    version::CLAP_VERSION,
};
use std::ffi::CStr;
use std::path::Path;

#[repr(C)]
pub struct PluginEntryDescriptor(clap_plugin_entry);

impl PluginEntryDescriptor {
    #[inline]
    pub fn from_raw(raw: &clap_plugin_entry) -> &Self {
        unsafe { ::core::mem::transmute(raw) }
    }

    #[inline]
    pub fn as_raw(&self) -> &clap_plugin_entry {
        &self.0
    }
}

pub trait PluginEntry: Sized {
    fn init(_plugin_path: &Path) {}
    fn de_init() {}

    fn plugin_count() -> u32;
    fn plugin_descriptor(index: u32) -> Option<&'static PluginDescriptor>;
    fn create_plugin<'a>(host_info: HostInfo<'a>, plugin_id: &[u8]) -> Option<PluginInstance<'a>>;

    const DESCRIPTOR: PluginEntryDescriptor = PluginEntryDescriptor(clap_plugin_entry {
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
    let path = CStr::from_ptr(plugin_path).to_bytes();
    let path = ::core::str::from_utf8(path).unwrap(); // TODO: unsafe unwrap
    E::init(Path::new(path));
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
    let host_info = HostInfo {
        inner: clap_host.as_ref().unwrap(),
    }; // TODO: unsafe unwrap

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
