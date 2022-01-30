//! Audio-processing events and related utilities.
//!
//! Events notify a plugin's Audio Processor of anything that may change its audio output, such as
//! note [on](crate::events::Event::NoteOn)/[off](crate::events::Event::NoteOff) events,
//! [parameter changes](crate::events::Event::ParamValue), [MIDI events](crate::events::Event::Midi),
//! and more.
//!
//! All events in CLAP are sample-accurate time-stamped events ([`TimestampedEvent`](crate::events::TimestampedEvent)).
//! They are provided to the plugin's audio processor alongside the audio buffers through [`EventList`s](crate::events::EventList)
//! (see the plugin's `process` method).

use bitflags::bitflags;
use clap_sys::events::clap_event_header;
use std::cmp::Ordering;
use std::marker::PhantomData;

mod list;
pub use list::*;

mod spaces;

pub use spaces::*;

pub mod event_types;

pub unsafe trait Event<'a>: Sized + 'a {
    const TYPE_ID: u16;
    type EventSpace: EventSpace<'a>;

    #[inline]
    fn raw_header(&self) -> &clap_event_header {
        unsafe { &*(self as *const Self as *const _) }
    }

    #[inline]
    fn header(&self) -> &EventHeader<Self> {
        unsafe { EventHeader::from_raw(self.raw_header()) }
    }

    #[inline]
    fn as_unknown(&self) -> &UnknownEvent<'a> {
        unsafe { UnknownEvent::from_raw(self.raw_header()) }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Eq, Debug)]
pub struct EventHeader<E = ()> {
    inner: clap_event_header,
    _event: PhantomData<E>,
}

impl<E> EventHeader<E> {
    #[inline]
    pub const unsafe fn from_raw(header: &clap_event_header) -> &Self {
        // SAFETY: EventHeader is repr(C) and ABI compatible
        &*(header as *const _ as *const _)
    }

    #[inline]
    pub const fn into_raw(self) -> clap_event_header {
        self.inner
    }

    #[inline]
    pub const fn size(&self) -> u32 {
        self.inner.size
    }

    #[inline]
    pub const fn type_id(&self) -> u16 {
        self.inner.type_
    }

    #[inline]
    pub const fn time(&self) -> u32 {
        self.inner.time
    }

    #[inline]
    pub const fn flags(&self) -> EventFlags {
        EventFlags::from_bits_truncate(self.inner.flags)
    }
}

impl EventHeader<()> {
    #[inline]
    pub const fn space_id(&self) -> Option<EventSpaceId> {
        EventSpaceId::new(self.inner.space_id)
    }
}

impl<'a, E: Event<'a>> EventHeader<E> {
    #[inline]
    pub fn new_for_space(
        space_id: EventSpaceId<E::EventSpace>,
        time: u32,
        flags: EventFlags,
    ) -> Self {
        Self {
            inner: clap_event_header {
                size: ::core::mem::size_of::<E>() as u32,
                time,
                space_id: space_id.id(),
                type_: E::TYPE_ID,
                flags: flags.bits,
            },
            _event: PhantomData,
        }
    }

    #[inline]
    pub fn space_id(&self) -> EventSpaceId<E::EventSpace> {
        // SAFETY: the EventHeader type guarantees the space_id correctness
        unsafe { EventSpaceId::new_unchecked(self.inner.space_id) }
    }
}

impl<'a, E: Event<'a, EventSpace = CoreEventSpace<'a>>> EventHeader<E> {
    #[inline]
    pub fn new(time: u32, flags: EventFlags) -> Self {
        Self::new_for_space(EventSpaceId::core(), time, flags)
    }
}

use crate::events::core::CoreEventSpace;
use clap_sys::events::{
    CLAP_EVENT_BEGIN_ADJUST, CLAP_EVENT_END_ADJUST, CLAP_EVENT_IS_LIVE, CLAP_EVENT_SHOULD_RECORD,
};

bitflags! {
    #[repr(C)]
    pub struct EventFlags: u32 {
        const IS_LIVE = CLAP_EVENT_IS_LIVE;
        const BEGIN_ADJUST = CLAP_EVENT_BEGIN_ADJUST;
        const END_ADJUST = CLAP_EVENT_END_ADJUST;
        const SHOULD_RECORD = CLAP_EVENT_SHOULD_RECORD;
    }
}

impl<E> PartialEq for EventHeader<E> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.time == other.inner.time
    }
}

impl<E> PartialOrd for EventHeader<E> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.time.partial_cmp(&other.inner.time)
    }
}

#[repr(C)]
pub struct UnknownEvent<'a> {
    header: EventHeader,
    _sysex_lifetime: PhantomData<&'a u8>,
}

impl<'a> UnknownEvent<'a> {
    #[inline]
    pub const unsafe fn from_raw(header: &clap_event_header) -> &Self {
        // SAFETY: EventHeader is repr(C) and ABI compatible
        &*(header as *const _ as *const _)
    }

    #[inline]
    pub const fn as_raw(&self) -> &clap_event_header {
        &self.header.inner
    }

    #[inline]
    pub const fn header(&self) -> &EventHeader {
        &self.header
    }

    #[inline]
    pub fn as_event_for_space<E: Event<'a>>(
        &self,
        space_id: EventSpaceId<E::EventSpace>,
    ) -> Option<&E> {
        let raw = &self.header.inner;
        if raw.space_id != space_id.id()
            || raw.type_ != E::TYPE_ID
            || raw.size != ::core::mem::size_of::<E>() as u32
        {
            return None;
        }

        // SAFETY: this type guarantees the header is followed by event data, and we just checked the space_id, type and size fields
        Some(unsafe { self.as_event_unchecked() })
    }

    #[inline]
    pub fn as_event<E: Event<'a, EventSpace = CoreEventSpace<'a>>>(&self) -> Option<&E> {
        self.as_event_for_space(EventSpaceId::core())
    }

    #[inline]
    pub unsafe fn as_event_unchecked<E: Event<'a>>(&self) -> &E {
        &*(self as *const _ as *const E)
    }

    #[inline]
    pub fn as_core_event(&self) -> Option<CoreEventSpace> {
        self.as_event_space(EventSpaceId::core())
    }

    #[inline]
    pub fn as_event_space<'s, S: EventSpace<'s>>(&'s self, space_id: EventSpaceId<S>) -> Option<S>
    where
        'a: 's,
    {
        if space_id.id() != self.header.inner.space_id {
            return None;
        }

        unsafe { S::from_unknown(self) }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: any data can be safely transmuted to a slice of bytes. This type also ensures
        // the size field is correct
        unsafe {
            ::core::slice::from_raw_parts(
                self as *const _ as *const _,
                self.header.inner.size as usize,
            )
        }
    }

    #[inline]
    pub unsafe fn from_bytes(bytes: &[u8]) -> &UnknownEvent {
        &*(bytes.as_ptr() as *const _)
    }
}
