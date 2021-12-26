use crate::events::event_match::EventTarget;
use clap_sys::events::clap_event_note_mask;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct NoteMaskEvent {
    inner: clap_event_note_mask,
}

impl NoteMaskEvent {
    #[inline]
    pub fn new(port_index: EventTarget<u16>, note_mask: u16, root_note: u8) -> Self {
        Self {
            inner: clap_event_note_mask {
                port_index: port_index.to_raw(),
                note_mask,
                root_note,
            },
        }
    }

    #[inline]
    pub fn from_raw(raw: clap_event_note_mask) -> Self {
        Self { inner: raw }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_note_mask {
        self.inner
    }
}

impl PartialEq for NoteMaskEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.port_index == other.inner.port_index
            && self.inner.note_mask == other.inner.note_mask
            && self.inner.root_note == other.inner.root_note
    }
}

impl Debug for NoteMaskEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        struct Mask(u16);

        impl Debug for Mask {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                core::fmt::Binary::fmt(&self.0, f)
            }
        }

        f.debug_struct("NoteMaskEvent")
            .field("port_index", &self.inner.port_index)
            .field("root_note", &self.inner.root_note)
            .field("note_mask", &Mask(self.inner.note_mask))
            .finish()
    }
}
