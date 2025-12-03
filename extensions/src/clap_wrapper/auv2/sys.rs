#![allow(non_camel_case_types)]

use core::ffi::{CStr, c_char};

pub const CLAP_PLUGIN_FACTORY_INFO_AUV2: &CStr = c"clap.plugin-factory-info-as-auv2.draft0";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_info_as_auv2 {
    pub au_type: [u8; 5],
    pub au_subt: [u8; 5],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct clap_plugin_factory_as_auv2 {
    pub manufacturer_code: *const c_char,
    pub manufacturer_name: *const c_char,

    pub get_auv2_info: Option<
        unsafe extern "C" fn(
            factory: *mut clap_plugin_factory_as_auv2,
            index: u32,
            info: *mut clap_plugin_info_as_auv2,
        ) -> bool,
    >,
}
