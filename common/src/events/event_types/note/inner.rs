#![deny(missing_docs)]

use crate::events::{Event, EventFlags, EventHeader, Pckn};
use clap_sys::events::clap_event_note;
use std::fmt::Formatter;
use std::marker::PhantomData;

/// Generic CLAP note event.
///
/// Notes and voices in CLAP are addressed by a 4‑value tuple
/// `(port, channel, key, note_id)`, called a PCKN in the CLAP headers.
///
/// In Clack, this concept is represented by the [`Pckn`] type, which
/// provides a safe abstraction over raw values and wildcards via the
/// [`Match`] enum.
///
/// Fields in the raw event are either `0` or greater, or `-1` to indicate
/// a match on any value as a wildcard. When using [`Pckn`], wildcards are
/// expressed with [`Match::All`] instead.
///
/// # Example
/// Handling a note or parameter event and checking its [`Pckn`] target:
/// ```no_run
/// use clack_common::events::{UnknownEvent, spaces::CoreEventSpace};
///
/// fn handle_event(event: &UnknownEvent) {
///     if let Some(CoreEventSpace::ParamValue(ev)) = event.as_core_event() {
///         if ev.pckn().matches_all() {
///             // Global modulation
///         } else {
///             // Per‑voice modulation
///         }
///     }
/// }
/// ```
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
                port_index: pckn.raw_port_index(),
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

    pub(crate) fn fmt(&self, f: &mut Formatter<'_>, event_name: &'static str) -> core::fmt::Result {
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
        crate::events::impl_event_pckn!(self.inner.inner);

        /// Returns a shared reference to the underlying raw, C-FFI compatible event struct.
        #[inline]
        pub const fn as_raw(&self) -> &clap_event_note {
            &self.inner.inner
        }

        /// Returns a mutable reference to the underlying raw, C-FFI compatible event struct.
        #[inline]
        pub const fn as_raw_mut(&mut self) -> &mut clap_event_note {
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
            crate::events::ensure_event_matches::<Self>(&raw.header);

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
            crate::events::ensure_event_matches::<Self>(&raw.header);

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
        pub const fn from_raw_mut(raw: &mut clap_event_note) -> &mut Self {
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
