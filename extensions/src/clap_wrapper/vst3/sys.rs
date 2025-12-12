#![allow(non_camel_case_types)]

use clap_sys::plugin::clap_plugin;
use core::ffi::{CStr, c_char};

#[cfg(any(feature = "clack-host", feature = "clack-plugin"))]
pub const CLAP_PLUGIN_FACTORY_INFO_VST3: &CStr = c"clap.plugin-factory-info-as-vst3/0";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_info_as_vst3 {
    pub vendor: *const c_char,
    pub component_id: *const [u8; 16],
    pub features: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[cfg(any(feature = "clack-host", feature = "clack-plugin"))]
pub struct clap_plugin_factory_as_vst3 {
    pub vendor: *const c_char,
    pub vendor_url: *const c_char,
    pub email_contact: *const c_char,

    pub get_vst3_info: Option<
        unsafe extern "C" fn(
            factory: *mut clap_plugin_factory_as_vst3,
            index: u32,
        ) -> *const clap_plugin_info_as_vst3,
    >,
}

// SAFETY: everything here is read-only
unsafe impl Send for clap_plugin_factory_as_vst3 {}
// SAFETY: everything here is read-only
unsafe impl Sync for clap_plugin_factory_as_vst3 {}

pub const CLAP_PLUGIN_AS_VST3: &CStr = c"clap.plugin-info-as-vst3/0";
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_as_vst3 {
    pub get_num_midi_channels:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin, note_port: u32) -> u32>,
    pub supported_note_expressions: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
}
