//! This extension provides a way for the plugin to describe its current note ports.
//! If the plugin does not implement this extension, it won't have note input or output.
//! The plugin is only allowed to change its note ports configuration while it is deactivated.

use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clack_common::utils::ClapId;
use clap_sys::ext::note_ports::*;
use std::ffi::CStr;

/// The Plugin-side of the Note Ports extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginNotePorts(RawExtension<PluginExtensionSide, clap_plugin_note_ports>);

/// The Host-side of the Note Ports extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostNotePorts(RawExtension<HostExtensionSide, clap_host_note_ports>);

bitflags! {
    /// Flags to indicate what note port information has changed and needs to be rescanned by the host.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NotePortRescanFlags: u32 {
        /// Invalidates everything the host knows about parameters.
        /// This can only be used while the plugin is deactivated.
        const ALL = CLAP_NOTE_PORTS_RESCAN_ALL;

        /// The ports name did change, the host can scan them right away.
        const NAMES = CLAP_NOTE_PORTS_RESCAN_NAMES;
    }
}

bitflags! {
    /// A set of [`NoteDialect`]s.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NoteDialects: u32 {
        /// See [`NoteDialect::Clap`].
        const CLAP = CLAP_NOTE_DIALECT_CLAP;
        /// See [`NoteDialect::Midi`].
        const MIDI = CLAP_NOTE_DIALECT_MIDI;
        /// See [`NoteDialect::MidiMpe`].
        const MIDI_MPE = CLAP_NOTE_DIALECT_MIDI_MPE;
        /// See [`NoteDialect::Midi2`].
        const MIDI2 = CLAP_NOTE_DIALECT_MIDI2;
    }
}

impl NoteDialects {
    /// Checks if a dialect is supported by this set of dialects.
    #[inline]
    pub fn supports(&self, dialect: NoteDialect) -> bool {
        self.contains(dialect.into())
    }
}

/// Possible supported note dialects for a note port.
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum NoteDialect {
    /// Events like [`NoteOnEvent`](clack_common::events::event_types::NoteOnEvent),
    /// [`NoteOffEvent`](clack_common::events::event_types::NoteOffEvent),
    /// [`NoteChokeEvent`](clack_common::events::event_types::NoteChokeEvent),
    /// [`NoteEndEvent`](clack_common::events::event_types::NoteEndEvent), etc.
    Clap = CLAP_NOTE_DIALECT_CLAP,
    /// Events like [`MidiEvent`](clack_common::events::event_types::MidiEvent),
    /// [`MidiSysExEvent`](clack_common::events::event_types::MidiSysExEvent).
    Midi = CLAP_NOTE_DIALECT_MIDI,
    /// Same as [`Midi`](Self::Midi), but with additional MPE support.
    MidiMpe = CLAP_NOTE_DIALECT_MIDI_MPE,
    /// Events like [`Midi2Event`](clack_common::events::event_types::Midi2Event).
    Midi2 = CLAP_NOTE_DIALECT_MIDI2,
}

impl NoteDialect {
    /// Converts a raw [`clap_note_dialect`] value into a [`NoteDialect`].
    pub const fn from_raw(raw: clap_note_dialect) -> Option<Self> {
        match raw {
            CLAP_NOTE_DIALECT_CLAP => Some(Self::Clap),
            CLAP_NOTE_DIALECT_MIDI => Some(Self::Midi),
            CLAP_NOTE_DIALECT_MIDI_MPE => Some(Self::MidiMpe),
            CLAP_NOTE_DIALECT_MIDI2 => Some(Self::Midi2),
            _ => None,
        }
    }
}

impl From<NoteDialect> for NoteDialects {
    #[inline]
    fn from(d: NoteDialect) -> Self {
        NoteDialects::from_bits_truncate(d as u32)
    }
}

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginNotePorts {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_NOTE_PORTS];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostNotePorts {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_NOTE_PORTS];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// Metadata describing a single note port.
pub struct NotePortInfo<'a> {
    /// Stable identifier for the port.
    ///
    /// IDs are allowed to match across directions (i.e. an input port and an output port can both have the same id),
    /// but are required to be unique within each direction (2 input ports, both with the same id are not allowed)
    pub id: ClapId,

    /// Display name for the port. Stored as a UTF‑8 byte slice.
    ///
    /// > **tip**: use `b""` syntax to set this easily
    /// > ```rust,ignore
    /// > name = b"MyNotePort",
    /// > ```
    pub name: &'a [u8],

    /// A set of supported note dialects for this port.
    /// See [`NoteDialects`] and [`NoteDialect`] for more information.
    pub supported_dialects: NoteDialects,

    /// The preferred dialect for this port.
    /// The host should use this dialect when possible, but can fall back to any of the supported dialects.
    ///
    /// Must be contained in [`supported_dialects`](Self::supported_dialects) if it's set.
    pub preferred_dialect: Option<NoteDialect>,
}

impl<'a> NotePortInfo<'a> {
    /// Converts a raw [`clap_note_port_info`] into a [`NotePortInfo`].
    pub fn from_raw(raw: &'a clap_note_port_info) -> Option<Self> {
        Some(Self {
            id: ClapId::from_raw(raw.id)?,
            name: crate::utils::data_from_array_buf(&raw.name),
            supported_dialects: NoteDialects::from_bits_truncate(raw.supported_dialects),
            preferred_dialect: NoteDialect::from_raw(raw.preferred_dialect),
        })
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
