use crate::events::event_match::EventTarget;
use clap_sys::events::clap_event_note;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct NoteEvent {
    inner: clap_event_note,
}

impl NoteEvent {
    #[inline]
    pub fn new(
        port_index: EventTarget<u16>,
        key: EventTarget,
        channel: EventTarget,
        velocity: f64,
    ) -> Self {
        Self {
            inner: clap_event_note {
                port_index: port_index.to_raw(),
                key: key.to_raw(),
                channel: channel.to_raw(),
                velocity,
            },
        }
    }

    #[inline]
    pub fn port_index(&self) -> EventTarget<u16> {
        EventTarget::from_raw(self.inner.port_index)
    }

    #[inline]
    pub fn key(&self) -> EventTarget {
        EventTarget::from_raw(self.inner.key)
    }

    #[inline]
    pub fn channel(&self) -> EventTarget {
        EventTarget::from_raw(self.inner.channel)
    }

    #[inline]
    pub fn velocity(&self) -> f64 {
        self.inner.velocity
    }

    #[inline]
    pub fn from_raw(inner: clap_event_note) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_note {
        self.inner
    }
}

impl PartialEq for NoteEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.key == other.inner.key
            && self.inner.channel == other.inner.channel
            && self.inner.port_index == other.inner.port_index
            && self.inner.velocity == other.inner.velocity
    }
}

impl Debug for NoteEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoteEvent")
            .field("port_index", &self.inner.port_index)
            .field("channel", &self.inner.channel)
            .field("key", &self.inner.key)
            .field("velocity", &self.inner.velocity)
            .finish()
    }
}
