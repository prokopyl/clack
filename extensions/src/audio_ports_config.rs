#![deny(missing_docs)]

//! A way for plugins to describe possible ports configurations, and for the host to select one.
//!
//! A configuration ([`AudioPortsConfiguration`]) is a very simple description of the audio ports:
//! it describes the main input and output ports, and has a name that can be displayed to the user.
//!
//! After the plugin initialization, the host may scan the list of configurations and eventually
//! select one that fits the plugin context. The host can also let the user change configurations
//! at any time, e.g. via a menu.
//!
//! The host can only select a configuration if the plugin is deactivated.
//!
//! Plugins with very complex configuration possibilities that cannot be covered by this extension,
//! should instead let the user configure the ports from the plugin GUI, and then request a full
//! port rescan to the host.

use crate::audio_ports::AudioPortType;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clack_common::utils::ClapId;
use clap_sys::ext::audio_ports_config::*;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

/// The Plugin-side of the Audio Ports Configurations extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginAudioPortsConfig(
    RawExtension<PluginExtensionSide, clap_plugin_audio_ports_config>,
);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginAudioPortsConfig {
    const IDENTIFIER: &'static CStr = CLAP_EXT_AUDIO_PORTS_CONFIG;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// The Host-side of the Audio Ports Configurations extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostAudioPortsConfig(RawExtension<HostExtensionSide, clap_host_audio_ports_config>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostAudioPortsConfig {
    const IDENTIFIER: &'static CStr = CLAP_EXT_AUDIO_PORTS_CONFIG;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

#[derive(Copy, Clone, Debug)]
/// A specific Audio Configuration for the plugin.
pub struct AudioPortsConfiguration<'a> {
    /// The ID of the configuration.
    ///
    /// It has to be unique for this instance of the plugin.
    pub id: ClapId,
    /// A user-facing display name for the configuration.
    pub name: &'a [u8],

    /// The number of input ports this configuration exposes
    pub input_port_count: u32,

    /// The number of output ports this configuration exposes
    pub output_port_count: u32,

    /// Information about the main input Audio Port of this configuration, if it has one.
    pub main_input: Option<MainPortInfo<'a>>,

    /// Information about the main output Audio Port of this configuration, if it has one.
    pub main_output: Option<MainPortInfo<'a>>,
}

#[cfg(feature = "clack-host")]
impl<'a> AudioPortsConfiguration<'a> {
    /// # Safety
    ///
    /// User must make sure all fields are valid for the lifetime of 'a.
    unsafe fn from_raw(raw: &'a clap_audio_ports_config) -> Option<Self> {
        use crate::utils::data_from_array_buf;

        Some(Self {
            id: ClapId::from_raw(raw.id)?,
            name: data_from_array_buf(&raw.name),

            input_port_count: raw.input_port_count,
            output_port_count: raw.output_port_count,

            main_input: MainPortInfo::from_raw(
                raw.has_main_input,
                raw.main_input_channel_count,
                raw.main_input_port_type,
            ),
            main_output: MainPortInfo::from_raw(
                raw.has_main_output,
                raw.main_output_channel_count,
                raw.main_output_port_type,
            ),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
/// Information about a main port.
pub struct MainPortInfo<'a> {
    /// The number of channels of this port.
    pub channel_count: u32,
    /// The type of this port.
    pub port_type: Option<AudioPortType<'a>>,
}

#[cfg(feature = "clack-host")]
impl MainPortInfo<'_> {
    /// # Safety
    ///
    /// User must make sure port_type is either null or points to a NULL-terminated C string that
    /// is valid for the lifetime of 'a.
    unsafe fn from_raw(
        exists: bool,
        channel_count: u32,
        port_type: *const std::os::raw::c_char,
    ) -> Option<Self> {
        if !exists {
            return None;
        }

        Some(Self {
            channel_count,
            // SAFETY: upheld by caller
            port_type: unsafe { AudioPortType::from_raw(port_type) },
        })
    }
}

/// An error that can occur as a plugin selects a new port configuration
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct AudioPortConfigSelectError;

impl Display for AudioPortConfigSelectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to change plugin audio ports configuration.")
    }
}

impl Error for AudioPortConfigSelectError {}

#[cfg(feature = "clack-host")]
mod host;

#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;

#[cfg(feature = "clack-plugin")]
pub use plugin::*;
