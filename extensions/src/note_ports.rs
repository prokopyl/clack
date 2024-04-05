use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::note_ports::*;
use std::ffi::CStr;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginNotePorts(RawExtension<PluginExtensionSide, clap_plugin_note_ports>);

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostNotePorts(RawExtension<HostExtensionSide, clap_host_note_ports>);

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NotePortRescanFlags: u32 {
        const ALL = CLAP_NOTE_PORTS_RESCAN_ALL;
        const NAMES = CLAP_NOTE_PORTS_RESCAN_NAMES;
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NoteDialects: u32 {
        const CLAP = CLAP_NOTE_DIALECT_CLAP;
        const MIDI = CLAP_NOTE_DIALECT_MIDI;
        const MIDI_MPE = CLAP_NOTE_DIALECT_MIDI_MPE;
        const MIDI2 = CLAP_NOTE_DIALECT_MIDI2;
    }
}

impl NoteDialects {
    #[inline]
    pub fn supports(&self, dialect: NoteDialect) -> bool {
        self.contains(dialect.into())
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum NoteDialect {
    Clap = CLAP_NOTE_DIALECT_CLAP,
    Midi = CLAP_NOTE_DIALECT_MIDI,
    MidiMpe = CLAP_NOTE_DIALECT_MIDI_MPE,
    Midi2 = CLAP_NOTE_DIALECT_MIDI2,
}

impl NoteDialect {
    pub fn from_raw(raw: clap_note_dialect) -> Option<Self> {
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
    const IDENTIFIER: &'static CStr = CLAP_EXT_NOTE_PORTS;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostNotePorts {
    const IDENTIFIER: &'static CStr = CLAP_EXT_NOTE_PORTS;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

pub struct NotePortInfo<'a> {
    pub id: u32,
    pub name: &'a [u8],
    pub supported_dialects: NoteDialects,
    pub preferred_dialect: Option<NoteDialect>,
}

impl<'a> NotePortInfo<'a> {
    pub fn from_raw(raw: &'a clap_note_port_info) -> Self {
        Self {
            id: raw.id,
            name: crate::utils::data_from_array_buf(&raw.name),
            supported_dialects: NoteDialects::from_bits_truncate(raw.supported_dialects),
            preferred_dialect: NoteDialect::from_raw(raw.preferred_dialect),
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
