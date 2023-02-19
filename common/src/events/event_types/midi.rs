use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, EventHeader, UnknownEvent};
use clap_sys::events::{
    clap_event_midi, clap_event_midi2, clap_event_midi_sysex, CLAP_EVENT_MIDI, CLAP_EVENT_MIDI2,
    CLAP_EVENT_MIDI_SYSEX,
};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct MidiEvent {
    inner: clap_event_midi,
}

unsafe impl<'a> Event<'a> for MidiEvent {
    const TYPE_ID: u16 = CLAP_EVENT_MIDI;
    type EventSpace = CoreEventSpace<'a>;
}

impl<'a> AsRef<UnknownEvent<'a>> for MidiEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent<'a> {
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
    pub fn from_raw(raw: clap_event_midi) -> Self {
        Self { inner: raw }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_midi {
        self.inner
    }

    #[inline]
    pub fn port_index(&self) -> u16 {
        self.inner.port_index
    }

    #[inline]
    pub fn set_port_index(&mut self, port_index: u16) {
        self.inner.port_index = port_index;
    }
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
            .field("port_index", &self.inner.port_index)
            .field("data", &self.inner.data)
            .finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MidiSysExEvent<'buf> {
    inner: clap_event_midi_sysex,
    _buffer_lifetime: PhantomData<&'buf [u8]>,
}

unsafe impl<'buf> Event<'buf> for MidiSysExEvent<'buf> {
    const TYPE_ID: u16 = CLAP_EVENT_MIDI_SYSEX;
    type EventSpace = CoreEventSpace<'buf>;
}

impl<'buf> AsRef<UnknownEvent<'buf>> for MidiSysExEvent<'buf> {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent<'buf> {
        self.as_unknown()
    }
}

impl<'buf> MidiSysExEvent<'buf> {
    /// # Safety
    /// This function allows creating an event from an arbitrary lifetime.
    /// Users of this method must ensure that the sysex buffer is valid for requested lifetime
    #[inline]
    pub unsafe fn from_raw(raw: clap_event_midi_sysex) -> Self {
        Self {
            _buffer_lifetime: PhantomData,
            inner: raw,
        }
    }

    #[inline]
    pub fn new(header: EventHeader<Self>, port_index: u16, buffer: &'buf [u8]) -> Self {
        Self {
            _buffer_lifetime: PhantomData,
            inner: clap_event_midi_sysex {
                header: header.into_raw(),
                port_index,
                buffer: buffer.as_ptr(),
                size: buffer.len() as u32,
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
    pub fn data(&self) -> &'buf [u8] {
        // SAFETY: this struct ensures the buffer is valid and for the required lifetime
        unsafe { core::slice::from_raw_parts(self.inner.buffer, self.inner.size as usize) }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_midi_sysex {
        self.inner
    }
}

impl<'a> PartialEq for MidiSysExEvent<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.port_index == other.inner.port_index && self.data() == other.data()
    }
}

impl<'a> Eq for MidiSysExEvent<'a> {}

impl<'a> Debug for MidiSysExEvent<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MidiSysexEvent")
            .field("port_index", &self.inner.port_index)
            .field("data", &self.data())
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct Midi2Event {
    inner: clap_event_midi2,
}

unsafe impl<'a> Event<'a> for Midi2Event {
    const TYPE_ID: u16 = CLAP_EVENT_MIDI2;
    type EventSpace = CoreEventSpace<'a>;
}

impl<'a> AsRef<UnknownEvent<'a>> for Midi2Event {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent<'a> {
        self.as_unknown()
    }
}

impl Midi2Event {
    #[inline]
    pub fn data(&self) -> [u32; 4] {
        self.inner.data
    }

    #[inline]
    pub fn set_data(&mut self, data: [u32; 4]) {
        self.inner.data = data
    }

    #[inline]
    pub fn from_raw(raw: clap_event_midi2) -> Self {
        Self { inner: raw }
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
    pub fn into_raw(self) -> clap_event_midi2 {
        self.inner
    }
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
            .field("port_index", &self.inner.port_index)
            .field("data", &self.inner.data)
            .finish()
    }
}
