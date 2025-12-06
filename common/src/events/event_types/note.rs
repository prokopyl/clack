use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, Match, Pckn, UnknownEvent};
use clap_sys::events::*;

mod inner;
use inner::*;

/// A note key pressed event.
///
/// A `NoteOnEvent` with a velocity of `0.0` is valid and should not be
/// interpreted as a `NoteOffEvent`.
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteOnEvent {
    inner: NoteEvent<NoteOnEvent>,
}

/// A note key released event.
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteOffEvent {
    inner: NoteEvent<NoteOffEvent>,
}

/// An event that chokes the voice(s) of a note,
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteChokeEvent {
    inner: NoteEvent<NoteChokeEvent>,
}

/// An event sent by the plugin to the host to indicate that a note has finished playing.
///
/// The port, channel, key, and note_id are those given by the host in the `NoteOnEvent`.
/// This event is useful to help the host match the plugin's voice life time, especially when
/// using polyphonic modulations, as only the plugin knows when a voice is truly finished.
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
    pub const fn set_velocity(&mut self, velocity: f64) {
        self.inner.inner.velocity = velocity
    }

    #[inline]
    pub const fn with_velocity(mut self, velocity: f64) -> Self {
        self.inner.inner.velocity = velocity;
        self
    }

    impl_note_helpers!();
}

impl NoteOffEvent {
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
    pub const fn set_velocity(&mut self, velocity: f64) {
        self.inner.inner.velocity = velocity
    }

    #[inline]
    pub const fn with_velocity(mut self, velocity: f64) -> Self {
        self.inner.inner.velocity = velocity;
        self
    }

    impl_note_helpers!();
}

impl NoteChokeEvent {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn) -> Self {
        Self {
            inner: NoteEvent::new(time, pckn, 0.0),
        }
    }

    impl_note_helpers!();
}

impl NoteEndEvent {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn) -> Self {
        Self {
            inner: NoteEvent::new(time, pckn, 0.0),
        }
    }

    impl_note_helpers!();
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

impl_note_traits!(NoteOnEvent);
impl_note_traits!(NoteOffEvent);
impl_note_traits!(NoteChokeEvent);
impl_note_traits!(NoteEndEvent);
