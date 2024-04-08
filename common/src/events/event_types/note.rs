use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, Pckn, UnknownEvent};
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
}

impl NoteOffEvent {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn, velocity: f64) -> Self {
        Self {
            inner: NoteEvent::new(time, pckn, velocity),
        }
    }
}

impl NoteChokeEvent {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn) -> Self {
        Self {
            inner: NoteEvent::new(time, pckn, 0.0),
        }
    }
}

impl NoteEndEvent {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn) -> Self {
        Self {
            inner: NoteEvent::new(time, pckn, 0.0),
        }
    }
}

// SAFETY: this matches the type ID and event space
unsafe impl<'a> Event<'a> for NoteOnEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_ON;
    type EventSpace = CoreEventSpace<'a>;
}

// SAFETY: this matches the type ID and event space
unsafe impl<'a> Event<'a> for NoteOffEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_OFF;
    type EventSpace = CoreEventSpace<'a>;
}

// SAFETY: this matches the type ID and event space
unsafe impl<'a> Event<'a> for NoteChokeEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_CHOKE;
    type EventSpace = CoreEventSpace<'a>;
}

// SAFETY: this matches the type ID and event space
unsafe impl<'a> Event<'a> for NoteEndEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_END;
    type EventSpace = CoreEventSpace<'a>;
}

self::impl_note!(NoteOnEvent);
self::impl_note!(NoteOffEvent);
self::impl_note!(NoteChokeEvent);
self::impl_note!(NoteEndEvent);
