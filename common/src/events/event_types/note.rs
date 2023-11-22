use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, EventHeader, UnknownEvent};
use clap_sys::events::*;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteOnEvent(pub NoteEvent<Self>);

unsafe impl<'a> Event<'a> for NoteOnEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_ON;
    type EventSpace = CoreEventSpace<'a>;
}

impl<'a> AsRef<UnknownEvent<'a>> for NoteOnEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent<'a> {
        self.as_unknown()
    }
}

impl Debug for NoteOnEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_note_event(&self.0, f, "NoteOnEvent")
    }
}

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteOffEvent(pub NoteEvent<Self>);

unsafe impl<'a> Event<'a> for NoteOffEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_OFF;
    type EventSpace = CoreEventSpace<'a>;
}

impl<'a> AsRef<UnknownEvent<'a>> for NoteOffEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent<'a> {
        self.as_unknown()
    }
}

impl Debug for NoteOffEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_note_event(&self.0, f, "NoteOffEvent")
    }
}

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteChokeEvent(pub NoteEvent<Self>);

unsafe impl<'a> Event<'a> for NoteChokeEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_CHOKE;
    type EventSpace = CoreEventSpace<'a>;
}

impl<'a> AsRef<UnknownEvent<'a>> for NoteChokeEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent<'a> {
        self.as_unknown()
    }
}

impl Debug for NoteChokeEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_note_event(&self.0, f, "NoteChokeEvent")
    }
}

#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct NoteEndEvent(pub NoteEvent<Self>);

unsafe impl<'a> Event<'a> for NoteEndEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_END;
    type EventSpace = CoreEventSpace<'a>;
}

impl<'a> AsRef<UnknownEvent<'a>> for NoteEndEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent<'a> {
        self.as_unknown()
    }
}

impl Debug for NoteEndEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_note_event(&self.0, f, "NoteEndEvent")
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct NoteEvent<E> {
    inner: clap_event_note,
    _event: PhantomData<E>,
}

impl<E> NoteEvent<E> {
    #[inline]
    pub const fn new(
        header: EventHeader<E>,
        note_id: i32,
        port_index: i16,
        key: i16,
        channel: i16,
        velocity: f64,
    ) -> Self {
        Self {
            inner: clap_event_note {
                header: header.into_raw(),
                note_id,
                port_index,
                key,
                channel,
                velocity,
            },
            _event: PhantomData,
        }
    }

    #[inline]
    pub const fn header(&self) -> &EventHeader<E> {
        // SAFETY: this type guarantees the event header is valid
        unsafe { EventHeader::from_raw_unchecked(&self.inner.header) }
    }

    #[inline]
    pub const fn port_index(&self) -> i16 {
        self.inner.port_index
    }

    #[inline]
    pub fn set_port_index(&mut self, port_index: i16) {
        self.inner.port_index = port_index;
    }

    #[inline]
    pub const fn note_id(&self) -> i32 {
        self.inner.note_id
    }

    #[inline]
    pub const fn key(&self) -> i16 {
        self.inner.key
    }

    #[inline]
    pub const fn channel(&self) -> i16 {
        self.inner.channel
    }

    #[inline]
    pub const fn velocity(&self) -> f64 {
        self.inner.velocity
    }

    #[inline]
    pub const fn from_raw(inner: clap_event_note) -> Self {
        Self {
            inner,
            _event: PhantomData,
        }
    }

    #[inline]
    pub const fn into_raw(self) -> clap_event_note {
        self.inner
    }
}

impl<E> PartialEq for NoteEvent<E> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.key == other.inner.key
            && self.inner.channel == other.inner.channel
            && self.inner.port_index == other.inner.port_index
            && self.inner.velocity == other.inner.velocity
            && self.inner.note_id == other.inner.note_id
    }
}

impl<E> Debug for NoteEvent<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fmt_note_event(self, f, "NoteEvent")
    }
}

fn fmt_note_event<E>(
    event: &NoteEvent<E>,
    f: &mut Formatter<'_>,
    event_name: &'static str,
) -> core::fmt::Result {
    f.debug_struct(event_name)
        .field("header", event.header())
        .field("port_index", &event.inner.port_index)
        .field("channel", &event.inner.channel)
        .field("key", &event.inner.key)
        .field("velocity", &event.inner.velocity)
        .field("note_id", &event.inner.note_id)
        .finish()
}
