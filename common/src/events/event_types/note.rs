use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, Match, Pckn, UnknownEvent};
use clap_sys::events::*;

mod inner;
use inner::*;

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteOnEvent {
    inner: NoteEvent<NoteOnEvent>,
}
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteOffEvent {
    inner: NoteEvent<NoteOffEvent>,
}
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteChokeEvent {
    inner: NoteEvent<NoteChokeEvent>,
}
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteEndEvent {
    inner: NoteEvent<NoteEndEvent>,
}
impl NoteOnEvent {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn, velocity: f64) -> Self {
        Self {
            inner: NoteEvent::new(time, pckn, velocity),
        }
    }

    #[inline]
    pub const fn velocity(&self) -> f64 {
        self.inner.inner.velocity
    }

    #[inline]
    pub fn set_velocity(&mut self, velocity: f64) {
        self.inner.inner.velocity = velocity
    }

    #[inline]
    pub const fn with_velocity(mut self, velocity: f64) -> Self {
        self.inner.inner.velocity = velocity;
        self
    }

    self::impl_note_helpers!();
}

impl NoteOffEvent {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn, velocity: f64) -> Self {
        Self {
            inner: NoteEvent::new(time, pckn, velocity),
        }
    }

    self::impl_note_helpers!();
}

impl NoteChokeEvent {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn) -> Self {
        Self {
            inner: NoteEvent::new(time, pckn, 0.0),
        }
    }

    self::impl_note_helpers!();
}

impl NoteEndEvent {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn) -> Self {
        Self {
            inner: NoteEvent::new(time, pckn, 0.0),
        }
    }

    self::impl_note_helpers!();
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for NoteOnEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_ON;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for NoteOffEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_OFF;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for NoteChokeEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_CHOKE;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for NoteEndEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_END;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

self::impl_note_traits!(NoteOnEvent);
self::impl_note_traits!(NoteOffEvent);
self::impl_note_traits!(NoteChokeEvent);
self::impl_note_traits!(NoteEndEvent);
