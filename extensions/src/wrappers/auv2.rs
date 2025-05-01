#![allow(non_camel_case_types)]

use clack_plugin::factory::Factory;
use core::ffi::c_char;
use core::ffi::CStr;
use std::panic::{catch_unwind, AssertUnwindSafe};

const CLAP_PLUGIN_FACTORY_INFO_AUV2: &CStr = c"clap.plugin-factory-info-as-auv2.draft0";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct clap_plugin_info_as_auv2 {
    au_type: [u8; 5],
    au_subt: [u8; 5],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct clap_plugin_factory_as_auv2 {
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

// SAFETY: everything here is read-only
unsafe impl Send for clap_plugin_factory_as_auv2 {}
// SAFETY: everything here is read-only
unsafe impl Sync for clap_plugin_factory_as_auv2 {}

pub struct PluginInfoAsAuv2 {
    inner: clap_plugin_info_as_auv2,
}

impl PluginInfoAsAuv2 {
    #[inline]
    pub fn new(au_type: &str, au_subt: &str) -> Self {
        let mut inner = clap_plugin_info_as_auv2 {
            au_subt: [0; 5],
            au_type: [0; 5],
        };

        let au_subt_len = au_type.len().min(5);
        let au_type_len = au_type.len().min(5);

        inner
            .au_subt
            .copy_from_slice(&au_subt.as_bytes()[..au_subt_len]);
        inner
            .au_type
            .copy_from_slice(&au_type.as_bytes()[..au_type_len]);

        Self { inner }
    }
}

pub trait PluginFactoryAsAUV2 {
    fn get_auv2_info(&self, index: u32) -> Option<PluginInfoAsAuv2>;
}

#[repr(C)]
pub struct PluginFactoryAsAUV2Wrapper<F> {
    raw: clap_plugin_factory_as_auv2,
    factory: F,
}

// SAFETY: PluginFactoryWrapper is #[repr(C)] with clap_plugin_factory_as_auv2 as its first field, and matches
// CLAP_PLUGIN_FACTORY_INFO_AUV2.
unsafe impl<F: PluginFactoryAsAUV2> Factory for PluginFactoryAsAUV2Wrapper<F> {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_INFO_AUV2;
}

impl<F: PluginFactoryAsAUV2> PluginFactoryAsAUV2Wrapper<F> {
    pub const fn new(
        manufacturer_code: &'static CStr,
        manufacturer_name: &'static CStr,
        factory: F,
    ) -> Self {
        Self {
            factory,
            raw: clap_plugin_factory_as_auv2 {
                get_auv2_info: Some(Self::get_auv2_info),
                manufacturer_code: manufacturer_code.as_ptr(),
                manufacturer_name: manufacturer_name.as_ptr(),
            },
        }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get_auv2_info(
        factory: *mut clap_plugin_factory_as_auv2,
        index: u32,
        info: *mut clap_plugin_info_as_auv2,
    ) -> bool {
        let Some(factory) = (factory as *const Self).as_ref() else {
            return false; // HOST_MISBEHAVING
        };

        let Ok(Some(info_data)) =
            catch_unwind(AssertUnwindSafe(|| factory.factory.get_auv2_info(index)))
        else {
            return false; // Either panicked or returned None.
        };

        info.write(info_data.inner);

        true
    }
}
