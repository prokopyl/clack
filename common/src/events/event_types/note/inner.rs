use crate::events::{Event, EventHeader, Pckn};
use clap_sys::events::clap_event_note;
use std::fmt::Formatter;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct NoteEvent<E> {
    pub inner: clap_event_note,
    _event: PhantomData<E>,
}

impl<'a, E: Event<EventSpace<'a> = CoreEventSpace<'a>>> NoteEvent<E> {
    #[inline]
    pub const fn new(time: u32, pckn: Pckn, velocity: f64) -> Self {
        Self {
            inner: clap_event_note {
                header: EventHeader::<E>::new(time).into_raw(),
                port_index: pckn.raw_port(),
                channel: pckn.raw_channel(),
                key: pckn.raw_key(),
                note_id: pckn.raw_note_id(),
                velocity,
            },
            _event: PhantomData,
        }
    }

    // TODO: move this to macro that makes const-compatible versions of trait methods.
    #[inline]
    pub const fn header(&self) -> &EventHeader<E> {
        // SAFETY: this type guarantees the event header is valid
        unsafe { EventHeader::from_raw_unchecked(&self.inner.header) }
    }

    #[inline]
    pub const fn from_raw(inner: &clap_event_note) -> Self {
        // TODO: panic if not matching

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
        #[inline]
        pub const fn pckn(&self) -> crate::events::Pckn {
            self.inner.pckn()
        }

        #[inline]
        pub fn set_pckn(&mut self, pckn: Pckn) {
            self.inner.set_pckn(pckn)
        }

        #[inline]
        pub fn with_pckn(mut self, pckn: Pckn) -> Self {
            self.inner.set_pckn(pckn);
            self
        }

        #[inline]
        pub const fn port(&self) -> Match<u16> {
            Match::<u16>::from_raw(self.inner.inner.port_index)
        }

        #[inline]
        pub fn set_port(&mut self, port: Match<u16>) {
            self.inner.inner.port_index = port.to_raw()
        }

        #[inline]
        pub const fn with_port(mut self, port: Match<u16>) -> Self {
            self.inner.inner.port_index = port.to_raw();
            self
        }

        #[inline]
        pub const fn channel(&self) -> Match<u16> {
            Match::<u16>::from_raw(self.inner.inner.channel)
        }

        #[inline]
        pub fn set_channel(&mut self, channel: Match<u16>) {
            self.inner.inner.channel = channel.to_raw();
        }

        #[inline]
        pub const fn with_channel(mut self, channel: Match<u16>) -> Self {
            self.inner.inner.channel = channel.to_raw();
            self
        }

        #[inline]
        pub const fn key(&self) -> Match<u16> {
            Match::<u16>::from_raw(self.inner.inner.key)
        }

        #[inline]
        pub fn set_key(&mut self, key: Match<u16>) {
            self.inner.inner.key = key.to_raw();
        }

        #[inline]
        pub const fn with_key(mut self, key: Match<u16>) -> Self {
            self.inner.inner.key = key.to_raw();
            self
        }

        #[inline]
        pub const fn note_id(&self) -> Match<u32> {
            Match::<u32>::from_raw(self.inner.inner.note_id)
        }

        #[inline]
        pub fn set_note_id(&mut self, note_id: Match<u32>) {
            self.inner.inner.note_id = note_id.to_raw();
        }

        #[inline]
        pub fn with_note_id(mut self, note_id: Match<u32>) -> Self {
            self.inner.inner.note_id = note_id.to_raw();
            self
        }

        #[inline]
        pub const fn as_raw(&self) -> &clap_event_note {
            &self.inner.inner
        }

        #[inline]
        pub fn as_raw_mut(&mut self) -> &mut clap_event_note {
            &mut self.inner.inner
        }

        #[inline]
        pub const fn from_raw(raw: &clap_event_note) -> Self {
            Self {
                inner: NoteEvent::from_raw(raw),
            }
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
