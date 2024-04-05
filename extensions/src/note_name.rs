#![deny(missing_docs)]

//! A way for plugins to list custom note names for hosts to display in e.g. a piano roll.

use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::note_name::*;
use std::ffi::CStr;

/// The Plugin-side of the Note Name extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginNoteName(RawExtension<PluginExtensionSide, clap_plugin_note_name>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginNoteName {
    const IDENTIFIER: &'static CStr = CLAP_EXT_NOTE_NAME;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

/// The Host-side of the Note Name extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostNoteName(RawExtension<HostExtensionSide, clap_host_note_name>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostNoteName {
    const IDENTIFIER: &'static CStr = CLAP_EXT_NOTE_NAME;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Copy, Clone, Debug)]
/// A Note's name.
pub struct NoteName<'a> {
    /// A user-facing display name for the note.
    pub name: &'a [u8],

    /// The Port this note name applies to, or `-1` if it applies to every key.
    pub port: i16,

    /// The MIDI Channel this note name applies to, or `-1` if it applies to every key.
    pub channel: i16,

    /// The Key this note name applies to, or `-1` if it applies to every key.
    pub key: i16,
}

impl<'a> NoteName<'a> {
    /// # Safety
    ///
    /// Users must ensure the given port info is valid.
    #[cfg(feature = "clack-host")]
    // TODO: make pub?
    unsafe fn from_raw(raw: &'a clap_note_name) -> Self {
        Self {
            name: crate::utils::data_from_array_buf(&raw.name),

            port: raw.port,
            key: raw.key,
            channel: raw.channel,
        }
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
