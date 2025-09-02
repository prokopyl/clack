use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clack_common::utils::{ClapId, Cookie};
use clap_sys::ext::params::*;
use std::ffi::CStr;

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamRescanFlags: u32 {
        const VALUES = CLAP_PARAM_RESCAN_VALUES;
        const INFO = CLAP_PARAM_RESCAN_INFO;
        const TEXT = CLAP_PARAM_RESCAN_TEXT;
        const ALL = CLAP_PARAM_RESCAN_ALL;
    }
}

impl ParamRescanFlags {
    /// Returns `true` if any of the given flags that are set imply that a plugin instance's restart
    /// is needed before params can be rescanned.
    #[inline]
    pub fn requires_restart(&self) -> bool {
        self.contains(Self::ALL)
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamClearFlags: u32 {
        const ALL = CLAP_PARAM_CLEAR_ALL;
        const AUTOMATIONS = CLAP_PARAM_CLEAR_AUTOMATIONS;
        const MODULATIONS = CLAP_PARAM_CLEAR_MODULATIONS;
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamInfoFlags: u32 {
        const IS_AUTOMATABLE = CLAP_PARAM_IS_AUTOMATABLE;
        const IS_AUTOMATABLE_PER_CHANNEL = CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL;
        const IS_AUTOMATABLE_PER_KEY = CLAP_PARAM_IS_AUTOMATABLE_PER_KEY;
        const IS_AUTOMATABLE_PER_NOTE_ID = CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID;
        const IS_AUTOMATABLE_PER_PORT = CLAP_PARAM_IS_AUTOMATABLE_PER_PORT;
        const IS_BYPASS = CLAP_PARAM_IS_BYPASS;
        const IS_HIDDEN = CLAP_PARAM_IS_HIDDEN;
        const IS_MODULATABLE = CLAP_PARAM_IS_MODULATABLE;
        const IS_MODULATABLE_PER_CHANNEL = CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL;
        const IS_MODULATABLE_PER_KEY = CLAP_PARAM_IS_MODULATABLE_PER_KEY;
        const IS_MODULATABLE_PER_NOTE_ID = CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID;
        const IS_MODULATABLE_PER_PORT = CLAP_PARAM_IS_MODULATABLE_PER_PORT;
        const IS_PERIODIC = CLAP_PARAM_IS_PERIODIC;
        const IS_READONLY = CLAP_PARAM_IS_READONLY;
        const IS_STEPPED = CLAP_PARAM_IS_STEPPED;
        const REQUIRES_PROCESS = CLAP_PARAM_REQUIRES_PROCESS;
        // NOTE: From the CLAP requirements: if CLAP_PARAM_IS_ENUM is set, then CLAP_PARAM_IS_STEPPED must
        // *also* be set, so just encode them together.
        const IS_ENUM = CLAP_PARAM_IS_ENUM | CLAP_PARAM_IS_STEPPED;
    }
}

impl ParamInfoFlags {
    pub const FLAGS_REQUIRING_INFO_RESCAN: Self =
        Self::from_bits_truncate(Self::IS_PERIODIC.bits() | Self::IS_HIDDEN.bits());
    pub const FLAGS_REQUIRING_FULL_RESCAN: Self = Self::from_bits_truncate(
        Self::IS_AUTOMATABLE.bits()
            | Self::IS_AUTOMATABLE_PER_NOTE_ID.bits()
            | Self::IS_AUTOMATABLE_PER_KEY.bits()
            | Self::IS_AUTOMATABLE_PER_CHANNEL.bits()
            | Self::IS_AUTOMATABLE_PER_PORT.bits()
            | Self::IS_MODULATABLE.bits()
            | Self::IS_MODULATABLE_PER_NOTE_ID.bits()
            | Self::IS_MODULATABLE_PER_KEY.bits()
            | Self::IS_MODULATABLE_PER_CHANNEL.bits()
            | Self::IS_MODULATABLE_PER_PORT.bits()
            | Self::IS_READONLY.bits()
            | Self::IS_BYPASS.bits()
            | Self::IS_STEPPED.bits(),
    );
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginParams(RawExtension<PluginExtensionSide, clap_plugin_params>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginParams {
    const IDENTIFIER: &'static CStr = CLAP_EXT_PARAMS;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostParams(RawExtension<HostExtensionSide, clap_host_params>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostParams {
    const IDENTIFIER: &'static CStr = CLAP_EXT_PARAMS;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}
pub struct ParamInfo<'a> {
    pub id: ClapId,
    pub flags: ParamInfoFlags,
    pub cookie: Cookie,
    pub name: &'a [u8],
    pub module: &'a [u8],
    pub min_value: f64,
    pub max_value: f64,
    pub default_value: f64,
}

impl<'a> ParamInfo<'a> {
    pub fn from_raw(raw: &'a clap_param_info) -> Option<Self> {
        Some(Self {
            id: ClapId::from_raw(raw.id)?,
            flags: ParamInfoFlags::from_bits_truncate(raw.flags),
            cookie: Cookie::from_raw(raw.cookie),
            name: crate::utils::data_from_array_buf(&raw.name),
            module: crate::utils::data_from_array_buf(&raw.module),
            min_value: raw.min_value,
            max_value: raw.max_value,
            default_value: raw.default_value,
        })
    }

    pub fn diff_for_rescan(&self, other: &ParamInfo) -> ParamRescanFlags {
        #[inline]
        fn flags_differ(
            a: ParamInfoFlags,
            b: ParamInfoFlags,
            flags_to_check: ParamInfoFlags,
        ) -> bool {
            a.intersection(flags_to_check) != b.intersection(flags_to_check)
        }

        let mut flags = ParamRescanFlags::empty();

        if self.name != other.name
            || self.module != other.module
            || flags_differ(
                self.flags,
                other.flags,
                ParamInfoFlags::FLAGS_REQUIRING_INFO_RESCAN,
            )
        {
            flags |= ParamRescanFlags::INFO;
        }

        if self.min_value != other.min_value
            || self.max_value != other.max_value
            || self.cookie != other.cookie
            || flags_differ(
                self.flags,
                other.flags,
                ParamInfoFlags::FLAGS_REQUIRING_FULL_RESCAN,
            )
        {
            flags |= ParamRescanFlags::ALL
        }

        flags
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
