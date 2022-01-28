use clap_sys::events::clap_event_note;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct NoteEvent {
    inner: clap_event_note,
}

impl NoteEvent {
    #[inline]
    pub fn new(port_index: i32, key: i32, channel: i32, velocity: f64) -> Self {
        Self {
            inner: clap_event_note {
                port_index,
                key,
                channel,
                velocity,
            },
        }
    }

    #[inline]
    pub fn port_index(&self) -> i32 {
        self.inner.port_index
    }

    #[inline]
    pub fn key(&self) -> i32 {
        self.inner.key
    }

    #[inline]
    pub fn channel(&self) -> i32 {
        self.inner.channel
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
