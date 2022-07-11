use crate::utils::{data_from_array_buf, from_bytes_until_nul};
use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtension, PluginExtension};
use clap_sys::ext::note_ports::*;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

#[repr(C)]
pub struct PluginNotePorts(
    clap_plugin_note_ports,
    PhantomData<*const clap_plugin_note_ports>,
);

#[repr(C)]
pub struct HostNotePorts(
    clap_host_note_ports,
    PhantomData<*const clap_host_note_ports>,
);

bitflags! {
    #[repr(C)]
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
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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

unsafe impl Extension for PluginNotePorts {
    const IDENTIFIER: &'static CStr = CLAP_EXT_NOTE_PORTS;
    type ExtensionType = PluginExtension;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginNotePorts {}
unsafe impl Sync for PluginNotePorts {}

unsafe impl Extension for HostNotePorts {
    const IDENTIFIER: &'static CStr = CLAP_EXT_NOTE_PORTS;
    type ExtensionType = HostExtension;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostNotePorts {}
unsafe impl Sync for HostNotePorts {}

#[derive(Clone)]
pub struct NotePortInfoBuffer {
    inner: MaybeUninit<clap_note_port_info>,
}

impl Default for NotePortInfoBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl NotePortInfoBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
        }
    }
}

pub struct NotePortInfoData<'a> {
    pub id: u32,
    pub name: &'a CStr,
    pub supported_dialects: NoteDialects,
    pub preferred_dialect: Option<NoteDialect>,
}

impl<'a> NotePortInfoData<'a> {
    unsafe fn try_from_raw(raw: &'a clap_note_port_info) -> Result<Self, ()> {
        Ok(Self {
            id: raw.id,
            name: from_bytes_until_nul(data_from_array_buf(&raw.name))?,
            supported_dialects: NoteDialects::from_bits_truncate(raw.supported_dialects),
            preferred_dialect: NoteDialect::from_raw(raw.preferred_dialect),
        })
    }
}

bitflags! {
    #[repr(C)]
    pub struct NotePortRescanFlags: u32 {
        const ALL = CLAP_NOTE_PORTS_RESCAN_ALL;
        const NAMES = CLAP_NOTE_PORTS_RESCAN_NAMES;
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
