use crate::events::event_match::NoteEventMatch;
use clap_sys::events::clap_event_note;

#[derive(Copy, Clone)]
pub struct NoteEvent {
    inner: clap_event_note,
}

impl NoteEvent {
    #[inline]
    pub fn new(
        port_index: i32,
        key: NoteEventMatch,
        channel: NoteEventMatch,
        velocity: f64,
    ) -> Self {
        Self {
            inner: clap_event_note {
                port_index,
                key: key.to_raw(),
                channel: channel.to_raw(),
                velocity,
            },
        }
    }

    #[inline]
    pub fn port_index(&self) -> i32 {
        self.inner.port_index
    }

    #[inline]
    pub fn key(&self) -> NoteEventMatch {
        NoteEventMatch::from_raw(self.inner.key)
    }

    #[inline]
    pub fn channel(&self) -> NoteEventMatch {
        NoteEventMatch::from_raw(self.inner.channel)
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
