#![deny(missing_docs)]

use crate::events::{Event, EventFlags, EventHeader, Pckn};
use clap_sys::events::clap_event_note;
use std::fmt::Formatter;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct NoteEvent<E> {
    pub inner: clap_event_note,
    _event: PhantomData<E>,
}

impl<E: for<'a> Event<EventSpace<'a> = CoreEventSpace<'a>>> NoteEvent<E> {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn, velocity: f64) -> Self {
        Self {
            inner: clap_event_note {
                header: EventHeader::<E>::new_core(time, EventFlags::empty()).into_raw(),
                port_index: pckn.raw_port(),
                channel: pckn.raw_channel(),
                key: pckn.raw_key(),
                note_id: pckn.raw_note_id(),
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
    pub const fn from_raw(inner: &clap_event_note) -> Self {
        Self {
            inner: *inner,
            _event: PhantomData,
        }
    }

    #[inline]
    pub const fn pckn(&self) -> Pckn {
        Pckn::from_raw(
            self.inner.port_index,
            self.inner.channel,
            self.inner.key,
            self.inner.note_id,
        )
    }

    #[inline]
    pub fn set_pckn(&mut self, pckn: Pckn) {
        self.inner.port_index = pckn.raw_port();
        self.inner.channel = pckn.raw_channel();
        self.inner.key = pckn.raw_key();
        self.inner.note_id = pckn.raw_note_id();
    }

    pub fn fmt(&self, f: &mut Formatter<'_>, event_name: &'static str) -> core::fmt::Result {
        f.debug_struct(event_name)
            .field("header", self.header())
            .field("port_index", &self.inner.port_index)
            .field("channel", &self.inner.channel)
            .field("key", &self.inner.key)
            .field("velocity", &self.inner.velocity)
            .field("note_id", &self.inner.note_id)
            .finish()
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

macro_rules! impl_note_helpers {
    () => {
        /// The [`Pckn`](crate::events::Pckn) tuple indicating which note(s) this note event targets.
        #[inline]
        pub const fn pckn(&self) -> crate::events::Pckn {
            self.inner.pckn()
        }

        /// Sets the [`Pckn`](crate::events::Pckn) tuple for this event.
        #[inline]
        pub fn set_pckn(&mut self, pckn: Pckn) {
            self.inner.set_pckn(pckn)
        }

        /// Sets the [`Pckn`](crate::events::Pckn) tuple for this event.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub fn with_pckn(mut self, pckn: Pckn) -> Self {
            self.inner.set_pckn(pckn);
            self
        }

        /// The index of the note port this event targets.
        ///
        /// This returns [`Match::All`] if this event targets all possible note ports.
        #[inline]
        pub const fn port_index(&self) -> Match<u16> {
            Match::<u16>::from_raw(self.inner.inner.port_index)
        }

        /// Sets the index of the note port this event targets.
        ///
        /// Use [`Match::All`] to target all possible note ports.
        #[inline]
        pub fn set_port_index(&mut self, port_index: Match<u16>) {
            self.inner.inner.port_index = port_index.to_raw()
        }

        /// Sets the index of the note port this event targets.
        ///
        /// Use [`Match::All`] to target all possible note ports.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub const fn with_port_index(mut self, port_index: Match<u16>) -> Self {
            self.inner.inner.port_index = port_index.to_raw();
            self
        }

        /// The note channel this event targets.
        ///
        /// This returns [`Match::All`] if this event targets all possible note channels.
        #[inline]
        pub const fn channel(&self) -> Match<u16> {
            Match::<u16>::from_raw(self.inner.inner.channel)
        }

        /// Sets the note channel this event targets.
        ///
        /// Use [`Match::All`] to target all possible channels.
        #[inline]
        pub fn set_channel(&mut self, channel: Match<u16>) {
            self.inner.inner.channel = channel.to_raw();
        }

        /// Sets the note channel this event targets.
        ///
        /// Use [`Match::All`] to target all possible channels.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub const fn with_channel(mut self, channel: Match<u16>) -> Self {
            self.inner.inner.channel = channel.to_raw();
            self
        }

        /// The key of the note(s) this event targets.
        ///
        /// This returns [`Match::All`] if this event targets all possible note keys.
        #[inline]
        pub const fn key(&self) -> Match<u16> {
            Match::<u16>::from_raw(self.inner.inner.key)
        }

        /// Sets the key of the note(s) this event targets.
        ///
        /// Use [`Match::All`] to target all possible note keys.
        #[inline]
        pub fn set_key(&mut self, key: Match<u16>) {
            self.inner.inner.key = key.to_raw();
        }

        /// Sets the key of the note(s) this event targets.
        ///
        /// Use [`Match::All`] to target all possible note keys.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub const fn with_key(mut self, key: Match<u16>) -> Self {
            self.inner.inner.key = key.to_raw();
            self
        }

        /// The specific ID of the Note this event targets.
        ///
        /// This returns [`Match::All`] if this event doesn't target a specific note, or doesn't
        /// provide a Note ID.
        #[inline]
        pub const fn note_id(&self) -> Match<u32> {
            Match::<u32>::from_raw(self.inner.inner.note_id)
        }

        /// Sets the specific ID of the Note this event targets.
        ///
        /// Use [`Match::All`] to not target a single specific note in particular.
        #[inline]
        pub fn set_note_id(&mut self, note_id: Match<u32>) {
            self.inner.inner.note_id = note_id.to_raw();
        }

        /// Sets the specific ID of the Note this event targets.
        ///
        /// Use [`Match::All`] to not target a single specific note in particular.
        ///
        /// This method takes and returns ownership of the event, allowing it to be used in a
        /// builder-style pattern.
        #[inline]
        pub fn with_note_id(mut self, note_id: Match<u32>) -> Self {
            self.inner.inner.note_id = note_id.to_raw();
            self
        }

        /// Returns a shared reference to the underlying raw, C-FFI compatible event struct.
        #[inline]
        pub const fn as_raw(&self) -> &clap_event_note {
            &self.inner.inner
        }

        /// Returns a mutable reference to the underlying raw, C-FFI compatible event struct.
        #[inline]
        pub fn as_raw_mut(&mut self) -> &mut clap_event_note {
            &mut self.inner.inner
        }

        /// Creates a new note event of this type from a reference to a raw, C-FFI compatible event
        /// struct.
        ///
        /// # Panics
        ///
        /// This method will panic if the given event struct's header doesn't actually match
        /// the expected note event type.
        #[inline]
        pub const fn from_raw(raw: &clap_event_note) -> Self {
            crate::events::ensure_event_matches_const::<Self>(&raw.header);

            Self {
                inner: NoteEvent::from_raw(raw),
            }
        }

        /// Creates a reference to a note event of this type from a reference to a raw,
        /// C-FFI compatible event struct.
        ///
        /// # Panics
        ///
        /// This method will panic if the given event struct's header doesn't actually match
        /// the expected note event type.
        #[inline]
        pub const fn from_raw_ref(raw: &clap_event_note) -> &Self {
            crate::events::ensure_event_matches_const::<Self>(&raw.header);

            // SAFETY: This type is #[repr(C)]-compatible with clap_event_note
            unsafe { &*(raw as *const clap_event_note as *const Self) }
        }

        /// Creates a mutable reference to a note event of this type from a reference to a raw,
        /// C-FFI compatible event struct.
        ///
        /// # Panics
        ///
        /// This method will panic if the given event struct's header doesn't actually match
        /// the expected note event type.
        #[inline]
        pub fn from_raw_mut(raw: &mut clap_event_note) -> &mut Self {
            crate::events::ensure_event_matches::<Self>(&raw.header);

            // SAFETY: This type is #[repr(C)]-compatible with clap_event_note
            unsafe { &mut *(raw as *mut clap_event_note as *mut Self) }
        }
    };
}

macro_rules! impl_note_traits {
    ($type:ty) => {
        const _: () = {
            impl AsRef<UnknownEvent> for $type {
                #[inline]
                fn as_ref(&self) -> &UnknownEvent {
                    self.as_unknown()
                }
            }

            impl std::fmt::Debug for $type {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.inner.fmt(f, stringify!($type))
                }
            }
        };
    };
}

use crate::events::spaces::CoreEventSpace;
pub(crate) use {impl_note_helpers, impl_note_traits};
