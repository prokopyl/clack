use crate::events::EventHeader;
use clap_sys::events::{clap_event_midi, clap_event_midi_sysex};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct MidiEvent {
    inner: clap_event_midi,
}

impl MidiEvent {
    #[inline]
    pub fn from_raw(raw: clap_event_midi) -> Self {
        Self { inner: raw }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_midi {
        self.inner
    }
}

impl PartialEq for MidiEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.data == other.inner.data && self.inner.port_index == other.inner.port_index
    }
}

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
pub struct MidiSysexEvent<'buf> {
    inner: clap_event_midi_sysex,
    _buffer_lifetime: PhantomData<&'buf [u8]>,
}

impl<'buf> MidiSysexEvent<'buf> {
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
    pub fn data(&self) -> &'buf [u8] {
        // SAFETY: this struct ensures the buffer is valid and for the required lifetime
        unsafe { ::core::slice::from_raw_parts(self.inner.buffer, self.inner.size as usize) }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_midi_sysex {
        self.inner
    }
}

impl<'a> PartialEq for MidiSysexEvent<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.port_index == other.inner.port_index && self.data() == other.data()
    }
}

impl<'a> Debug for MidiSysexEvent<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MidiSysexEvent")
            .field("port_index", &self.inner.port_index)
            .field("data", &self.data())
            .finish()
    }
}
