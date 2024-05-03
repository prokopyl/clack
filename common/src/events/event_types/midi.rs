use crate::events::helpers::impl_event_helpers;
use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, EventHeader, UnknownEvent};
use crate::utils::slice_from_external_parts;
use clap_sys::events::{
    clap_event_midi, clap_event_midi2, clap_event_midi_sysex, CLAP_EVENT_MIDI, CLAP_EVENT_MIDI2,
    CLAP_EVENT_MIDI_SYSEX,
};
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct MidiEvent {
    inner: clap_event_midi,
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for MidiEvent {
    const TYPE_ID: u16 = CLAP_EVENT_MIDI;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

impl AsRef<UnknownEvent> for MidiEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self.as_unknown()
    }
}

impl MidiEvent {
    #[inline]
    pub fn new(header: EventHeader<Self>, port_index: u16, data: [u8; 3]) -> Self {
        Self {
            inner: clap_event_midi {
                header: header.into_raw(),
                port_index,
                data,
            },
        }
    }

    #[inline]
    pub fn data(&self) -> [u8; 3] {
        self.inner.data
    }

    #[inline]
    pub fn set_data(&mut self, data: [u8; 3]) {
        self.inner.data = data
    }

    #[inline]
    pub fn with_data(mut self, data: [u8; 3]) -> Self {
        self.inner.data = data;
        self
    }

    #[inline]
    pub fn port_index(&self) -> u16 {
        self.inner.port_index
    }

    #[inline]
    pub fn set_port_index(&mut self, port_index: u16) {
        self.inner.port_index = port_index
    }

    #[inline]
    pub fn with_port_index(mut self, port_index: u16) -> Self {
        self.inner.port_index = port_index;
        self
    }

    impl_event_helpers!(clap_event_midi);
}

impl PartialEq for MidiEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.data == other.inner.data && self.inner.port_index == other.inner.port_index
    }
}

impl Eq for MidiEvent {}

impl Debug for MidiEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MidiEvent")
            .field("header", &self.header())
            .field("port_index", &self.inner.port_index)
            .field("data", &self.inner.data)
            .finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MidiSysExEvent {
    inner: clap_event_midi_sysex,
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for MidiSysExEvent {
    const TYPE_ID: u16 = CLAP_EVENT_MIDI_SYSEX;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

impl AsRef<UnknownEvent> for MidiSysExEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self.as_unknown()
    }
}

impl MidiSysExEvent {
    #[inline]
    pub fn new(header: EventHeader<Self>, port_index: u16, data: &[u8]) -> Self {
        Self {
            inner: clap_event_midi_sysex {
                header: header.into_raw(),
                port_index,
                buffer: data.as_ptr(),
                size: data.len() as u32,
            },
        }
    }

    #[inline]
    pub fn port_index(&self) -> u16 {
        self.inner.port_index
    }

    #[inline]
    pub fn set_port_index(&mut self, port_index: u16) {
        self.inner.port_index = port_index;
    }

    #[inline]
    pub fn with_port_index(mut self, port_index: u16) -> Self {
        self.inner.port_index = port_index;
        self
    }

    #[inline]
    pub fn buffer_ptr(&self) -> *const u8 {
        self.inner.buffer
    }

    #[inline]
    pub fn buffer_size(&self) -> u32 {
        self.inner.size
    }

    /// # Safety
    ///
    /// Users *must* ensure that the buffer lives long enough.
    /// As a plugin, host-provided buffers are guaranteed to live at least as long as the current
    /// method call (e.g. `process` or `flush`).
    ///
    /// As a host, plugin-provided buffers usually live at least until the next plugin call from the
    /// same thread.
    #[inline]
    pub unsafe fn data<'a>(&self) -> &'a [u8] {
        // SAFETY: this struct ensures the buffer is valid, and the user enforces the lifetime
        unsafe { slice_from_external_parts(self.inner.buffer, self.inner.size as usize) }
    }

    impl_event_helpers!(clap_event_midi_sysex);
}

impl PartialEq for MidiSysExEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.port_index == other.inner.port_index
            && self.buffer_size() == other.buffer_size()
            && self.buffer_ptr() == other.buffer_ptr()
    }
}

impl Eq for MidiSysExEvent {}

impl Debug for MidiSysExEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MidiSysexEvent")
            .field("header", &self.header())
            .field("port_index", &self.inner.port_index)
            .field("buffer_size", &self.buffer_size())
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct Midi2Event {
    inner: clap_event_midi2,
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for Midi2Event {
    const TYPE_ID: u16 = CLAP_EVENT_MIDI2;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

impl AsRef<UnknownEvent> for Midi2Event {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self.as_unknown()
    }
}

impl Midi2Event {
    #[inline]
    pub fn new(header: EventHeader<Self>, port_index: u16, data: [u32; 4]) -> Self {
        Self {
            inner: clap_event_midi2 {
                header: header.into_raw(),
                port_index,
                data,
            },
        }
    }

    #[inline]
    pub fn data(&self) -> [u32; 4] {
        self.inner.data
    }

    #[inline]
    pub fn set_data(&mut self, data: [u32; 4]) {
        self.inner.data = data
    }

    #[inline]
    pub fn with_data(mut self, data: [u32; 4]) -> Self {
        self.inner.data = data;
        self
    }

    #[inline]
    pub fn port_index(&self) -> u16 {
        self.inner.port_index
    }

    #[inline]
    pub fn set_port_index(&mut self, port_index: u16) {
        self.inner.port_index = port_index;
    }

    #[inline]
    pub fn with_port_index(mut self, port_index: u16) -> Self {
        self.inner.port_index = port_index;
        self
    }

    impl_event_helpers!(clap_event_midi2);
}

impl PartialEq for Midi2Event {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.data == other.inner.data && self.inner.port_index == other.inner.port_index
    }
}

impl Eq for Midi2Event {}

impl Debug for Midi2Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Midi2Event")
            .field("header", &self.header())
            .field("port_index", &self.inner.port_index)
            .field("data", &self.inner.data)
            .finish()
    }
}
