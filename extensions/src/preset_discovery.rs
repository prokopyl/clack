use clack_common::factory::{Factory, RawFactoryPointer};
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
