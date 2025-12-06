use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clack_common::utils::{ClapId, Cookie};
use clap_sys::ext::params::*;
use std::ffi::CStr;

bitflags! {
    /// Flags to indicate what parameter information has changed and needs to be rescanned by the host.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamRescanFlags: u32 {
        /// The parameter values have changed, e.g. after loading a preset.
        /// The host will scan all the parameter values.
        /// The host will not record those changes as automation points.
        const VALUES = CLAP_PARAM_RESCAN_VALUES;

        /// The parameter's info has changed (e.g. name, module, ranges).
        const INFO = CLAP_PARAM_RESCAN_INFO;

        /// The parameter's value to text conversion has changed.
        const TEXT = CLAP_PARAM_RESCAN_TEXT;

        /// Invalidates everything the host knows about parameters.
        /// This can only be used while the plugin is deactivated.
        const ALL = CLAP_PARAM_RESCAN_ALL;
    }
}

impl ParamRescanFlags {
    /// Returns `true` if any of the given flags that are set imply that a plugin instance's restart
    /// is needed before params can be rescanned.
    #[inline]
    pub const fn requires_restart(&self) -> bool {
        self.contains(Self::ALL)
    }
}

bitflags! {
    /// Flags to indicate what references to a parameter should be cleared by the host.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamClearFlags: u32 {
        /// Clears all possible references to a parameter, including automation and modulation.
        const ALL = CLAP_PARAM_CLEAR_ALL;
        /// Clears all automation for a parameter.
        const AUTOMATIONS = CLAP_PARAM_CLEAR_AUTOMATIONS;
        /// Clears all modulation for a parameter.
        const MODULATIONS = CLAP_PARAM_CLEAR_MODULATIONS;
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamInfoFlags: u32 {
        /// Automation can be recorded for this parameter.
        const IS_AUTOMATABLE = CLAP_PARAM_IS_AUTOMATABLE;

        /// This parameter supports per-channel automation.
        const IS_AUTOMATABLE_PER_CHANNEL = CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL;

        /// This parameter supports per-key automation.
        const IS_AUTOMATABLE_PER_KEY = CLAP_PARAM_IS_AUTOMATABLE_PER_KEY;

        /// This parameter supports per-note automation.
        const IS_AUTOMATABLE_PER_NOTE_ID = CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID;

        /// This parameter supports per-port automation.
        const IS_AUTOMATABLE_PER_PORT = CLAP_PARAM_IS_AUTOMATABLE_PER_PORT;

        /// This parameter is used to merge the plugin and host bypass button.
        /// It implies that the parameter is stepped, with `0.0` being bypass off, and `1.0` being bypass on.
        const IS_BYPASS = CLAP_PARAM_IS_BYPASS;

        /// This parameter should not be shown to the user, because it is currently not used.
        /// It is not necessary to process automation for this parameter.
        const IS_HIDDEN = CLAP_PARAM_IS_HIDDEN;

        /// This parameter supports modulation.
        const IS_MODULATABLE = CLAP_PARAM_IS_MODULATABLE;

        /// This parameter supports per-channel modulation.
        const IS_MODULATABLE_PER_CHANNEL = CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL;

        /// This parameter supports per-key modulation.
        const IS_MODULATABLE_PER_KEY = CLAP_PARAM_IS_MODULATABLE_PER_KEY;

        /// This parameter supports per-note modulation.
        const IS_MODULATABLE_PER_NOTE_ID = CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID;

        /// This parameter supports per-port modulation.
        const IS_MODULATABLE_PER_PORT = CLAP_PARAM_IS_MODULATABLE_PER_PORT;

        /// This parameter is periodic, e.g. a phase.
        const IS_PERIODIC = CLAP_PARAM_IS_PERIODIC;

        /// This parameter cannot be changed by the host.
        const IS_READONLY = CLAP_PARAM_IS_READONLY;

        /// This parameter is stepped (integer values only).
        /// If so, the double value is converted to integer using a cast (equivalent to `trunc`).
        const IS_STEPPED = CLAP_PARAM_IS_STEPPED;

        /// Any change to this parameter will affect the plugin's output and requires it to be
        /// processed via `process()` if the plugin is active.
        const REQUIRES_PROCESS = CLAP_PARAM_REQUIRES_PROCESS;

        /// This parameter represents an enumeration of discrete values.
        const IS_ENUM = CLAP_PARAM_IS_ENUM;
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
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_PARAMS];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostParams(RawExtension<HostExtensionSide, clap_host_params>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostParams {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_PARAMS];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}
/// Information about a parameter.
pub struct ParamInfo<'a> {
    /// A stable identifier for the parameter, which must never change.
    pub id: ClapId,
    /// Flags providing more information about the parameter.
    pub flags: ParamInfoFlags,
    /// An opaque pointer that can be used by the plugin to quickly access the parameter's data.
    ///
    /// This value is optional and set by the plugin. Its purpose is to provide fast access to the
    /// plugin parameter object by caching its pointer.
    ///
    /// The cookie is invalidated by a call to `clap_host_params.rescan(CLAP_PARAM_RESCAN_ALL)` or
    /// when the plugin is destroyed.
    pub cookie: Cookie,
    /// The display name of the parameter, e.g. "Volume".
    pub name: &'a [u8],
    /// The module path of the parameter, e.g. "Oscillators/Wavetable 1".
    /// The host can use `/` as a separator to show a tree-like structure.
    pub module: &'a [u8],
    /// The minimum plain value of the parameter.
    pub min_value: f64,
    /// The maximum plain value of the parameter.
    pub max_value: f64,
    /// The default plain value of the parameter.
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
