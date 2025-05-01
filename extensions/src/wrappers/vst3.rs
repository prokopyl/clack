#![allow(non_camel_case_types)]

use clack_common::extensions::{
    Extension, ExtensionImplementation, PluginExtensionSide, RawExtension,
    RawExtensionImplementation,
};
use clack_plugin::extensions::prelude::PluginWrapper;
use clack_plugin::factory::Factory;
use clack_plugin::prelude::Plugin;
use clap_sys::plugin::clap_plugin;
use core::ffi::{c_char, CStr};
use core::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};
// ===== Factory

const CLAP_PLUGIN_FACTORY_INFO_VST3: &CStr = c"clap.plugin-factory-info-as-vst3/0";

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct clap_plugin_info_as_vst3 {
    pub vendor: *const c_char,
    pub component_id: *const [u8; 16],
    pub features: *const c_char,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct clap_plugin_factory_as_vst3 {
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

#[derive(Debug, Copy, Clone)]
pub struct PluginInfoAsVST3<'a> {
    inner: clap_plugin_info_as_vst3,
    _lifetime: PhantomData<&'a CStr>,
}

// SAFETY: everything here is read-only
unsafe impl Send for clap_plugin_info_as_vst3 {}
// SAFETY: everything here is read-only
unsafe impl Sync for clap_plugin_info_as_vst3 {}

impl<'a> PluginInfoAsVST3<'a> {
    #[inline]
    pub const fn new(
        vendor: Option<&'a CStr>,
        component_id: Option<&'a [u8; 16]>,
        features: Option<&'a CStr>,
    ) -> Self {
        Self {
            _lifetime: PhantomData,
            inner: clap_plugin_info_as_vst3 {
                vendor: match vendor {
                    Some(v) => v.as_ptr(),
                    None => core::ptr::null(),
                },
                features: match features {
                    Some(v) => v.as_ptr(),
                    None => core::ptr::null(),
                },
                component_id: match component_id {
                    Some(v) => v,
                    None => core::ptr::null(),
                },
            },
        }
    }
}

pub trait PluginFactoryAsVST3 {
    fn get_vst3_info(&self, index: u32) -> Option<&PluginInfoAsVST3>;
}

#[repr(C)]
pub struct PluginFactoryAsVST3Wrapper<F> {
    raw: clap_plugin_factory_as_vst3,
    factory: F,
}

// SAFETY: PluginFactoryWrapper is #[repr(C)] with clap_plugin_factory_as_vst3 as its first field, and matches
// CLAP_PLUGIN_FACTORY_INFO_VST3.
unsafe impl<F: PluginFactoryAsVST3> Factory for PluginFactoryAsVST3Wrapper<F> {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_INFO_VST3;
}

impl<F: PluginFactoryAsVST3> PluginFactoryAsVST3Wrapper<F> {
    #[inline]
    pub const fn new(
        vendor: Option<&'static CStr>,
        vendor_url: Option<&'static CStr>,
        email_contact: Option<&'static CStr>,
        factory: F,
    ) -> Self {
        Self {
            factory,
            raw: clap_plugin_factory_as_vst3 {
                vendor: match vendor {
                    Some(v) => v.as_ptr(),
                    None => core::ptr::null(),
                },
                vendor_url: match vendor_url {
                    Some(v) => v.as_ptr(),
                    None => core::ptr::null(),
                },
                email_contact: match email_contact {
                    Some(v) => v.as_ptr(),
                    None => core::ptr::null(),
                },
                get_vst3_info: Some(Self::get_vst3_info),
            },
        }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get_vst3_info(
        factory: *mut clap_plugin_factory_as_vst3,
        index: u32,
    ) -> *const clap_plugin_info_as_vst3 {
        let Some(factory) = (factory as *const Self).as_ref() else {
            return core::ptr::null(); // HOST_MISBEHAVING
        };

        let Ok(Some(info)) =
            catch_unwind(AssertUnwindSafe(|| factory.factory.get_vst3_info(index)))
        else {
            return core::ptr::null(); // Either panicked or returned None.
        };

        &info.inner
    }
}

// ===== Extension

const CLAP_PLUGIN_AS_VST3: &CStr = c"clap.plugin-info-as-vst3/0";
#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct clap_plugin_as_vst3 {
    pub get_num_midi_channels:
        Option<unsafe extern "C" fn(plugin: *const clap_plugin, note_port: u32) -> u32>,
    pub supported_note_expressions: Option<unsafe extern "C" fn(plugin: *const clap_plugin) -> u32>,
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginAsVST3(RawExtension<PluginExtensionSide, clap_plugin_as_vst3>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginAsVST3 {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_AS_VST3;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

pub trait PluginAsVST3Impl {
    fn num_midi_channels(&self, note_port: u32) -> u32;
    fn supported_note_expressions(&self) -> u32;
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<P: Plugin> ExtensionImplementation<P> for PluginAsVST3
where
    for<'a> P::Shared<'a>: PluginAsVST3Impl,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_as_vst3 {
            get_num_midi_channels: Some(get_num_midi_channels::<P>),
            supported_note_expressions: Some(supported_note_expressions::<P>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get_num_midi_channels<P: Plugin>(
    plugin: *const clap_plugin,
    note_port: u32,
) -> u32
where
    for<'a> P::Shared<'a>: PluginAsVST3Impl,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.shared().num_midi_channels(note_port))
    })
    .unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn supported_note_expressions<P: Plugin>(plugin: *const clap_plugin) -> u32
where
    for<'a> P::Shared<'a>: PluginAsVST3Impl,
{
    PluginWrapper::<P>::handle(plugin, |plugin| {
        Ok(plugin.shared().supported_note_expressions())
    })
    .unwrap_or(0)
}
