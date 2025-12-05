#![deny(missing_docs)]

//! A way for plugins to list custom note names for hosts to display in e.g. a piano roll.

use clack_common::events::Match;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::note_name::*;
use clap_sys::string_sizes::CLAP_NAME_SIZE;
use std::ffi::CStr;

/// The Plugin-side of the Note Name extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginNoteName(RawExtension<PluginExtensionSide, clap_plugin_note_name>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginNoteName {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_NOTE_NAME];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// The Host-side of the Note Name extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostNoteName(RawExtension<HostExtensionSide, clap_host_note_name>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostNoteName {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_NOTE_NAME];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

#[derive(Copy, Clone, Debug)]
/// A Note's name.
pub struct NoteName<'a> {
    /// A user-facing display name for the note.
    pub name: &'a [u8],

    /// The Port this note name applies to, or `-1` if it applies to every port.
    pub port: Match<u16>,

    /// The MIDI Channel this note name applies to, or `-1` if it applies to every channel.
    pub channel: Match<u16>,

    /// The Key this note name applies to, or `-1` if it applies to every key.
    pub key: Match<u16>,
}

impl<'a> NoteName<'a> {
    /// Creates a new [`NoteName`] from a reference to the given raw C ABI-compatible buffer.
    pub fn from_raw(raw: &'a clap_note_name) -> Self {
        Self {
            name: crate::utils::data_from_array_buf(&raw.name),

            port: Match::<u16>::from_raw(raw.port),
            channel: Match::<u16>::from_raw(raw.channel),
            key: Match::<u16>::from_raw(raw.key),
        }
    }

    /// Creates a new raw C ABI-compatible note name buffer from this [`NoteName`].
    pub fn to_raw(&self) -> clap_note_name {
        let mut name = [0; CLAP_NAME_SIZE];
        // SAFETY: name is a valid pointer, as it comes from a &mut reference.
        unsafe { crate::utils::write_to_array_buf(&mut name, self.name) }

        clap_note_name {
            name,
            port: self.port.to_raw(),
            channel: self.channel.to_raw(),
            key: self.key.to_raw(),
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
