use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clack_common::factory::{Factory, RawFactoryPointer};
use clap_sys::ext::preset_load::*;
use clap_sys::factory::preset_discovery::*;
use std::ffi::CStr;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct PresetDiscoveryFactory<'a>(RawFactoryPointer<'a, clap_preset_discovery_factory>);

// SAFETY: clap_preset_discovery_factory is the CLAP type tied to CLAP_PRESET_DISCOVERY_FACTORY_ID and CLAP_PRESET_DISCOVERY_FACTORY_ID_COMPAT
unsafe impl<'a> Factory<'a> for PresetDiscoveryFactory<'a> {
    const IDENTIFIERS: &'static [&'static CStr] = &[
        CLAP_PRESET_DISCOVERY_FACTORY_ID,
        CLAP_PRESET_DISCOVERY_FACTORY_ID_COMPAT,
    ];
    type Raw = clap_preset_discovery_factory;

    #[inline]
    unsafe fn from_raw(raw: RawFactoryPointer<'a, Self::Raw>) -> Self {
        Self(raw)
    }
}

impl<'a> PresetDiscoveryFactory<'a> {
    /// Returns this factory as a raw pointer to its C-FFI compatible raw CLAP structure
    #[inline]
    pub const fn raw(&self) -> RawFactoryPointer<'a, clap_preset_discovery_factory> {
        self.0
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct PluginPresetLoad(RawExtension<PluginExtensionSide, clap_plugin_preset_load>);

// SAFETY: TODO
unsafe impl Extension for PluginPresetLoad {
    const IDENTIFIERS: &'static [&'static CStr] =
        &[CLAP_EXT_PRESET_LOAD, CLAP_EXT_PRESET_LOAD_COMPAT];
    type ExtensionSide = PluginExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: TODO
        unsafe { Self(raw.cast()) }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct HostPresetLoad(RawExtension<HostExtensionSide, clap_host_preset_load>);

// SAFETY: TODO
unsafe impl Extension for HostPresetLoad {
    const IDENTIFIERS: &'static [&'static CStr] =
        &[CLAP_EXT_PRESET_LOAD, CLAP_EXT_PRESET_LOAD_COMPAT];
    type ExtensionSide = HostExtensionSide;

    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: TODO
        unsafe { Self(raw.cast()) }
    }
}

#[cfg(feature = "clack-host")]
mod host;

#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;

#[cfg(feature = "clack-plugin")]
pub use plugin::*;

mod data;
mod descriptor;

pub use data::*;
pub use descriptor::*;
