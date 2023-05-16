use bitflags::bitflags;
use clap_sys::events::clap_event_header;
use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct EventHeader<E = ()> {
    inner: clap_event_header,
    _event: PhantomData<E>,
}

impl<E> EventHeader<E> {
    /// Gets a typed event header from a raw, untyped header, without performing any checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure the given header matches the given event type `E`.
    #[inline]
    pub const unsafe fn from_raw_unchecked(header: &clap_event_header) -> &Self {
        // SAFETY: EventHeader is repr(C) and ABI compatible
        &*(header as *const _ as *const _)
    }

    #[inline]
    pub const fn as_raw(&self) -> &clap_event_header {
        &self.inner
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
    pub const fn payload_size(&self) -> u32 {
        self.inner.size - (core::mem::size_of::<EventHeader>() as u32)
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
    /// Gets an untyped event header from a raw header.
    #[inline]
    pub const fn from_raw(header: &clap_event_header) -> &Self {
        // SAFETY: This EventHeader's type is (), i.e. untyped.
        unsafe { Self::from_raw_unchecked(header) }
    }

    #[inline]
    pub const fn space_id(&self) -> Option<EventSpaceId> {
        EventSpaceId::new(self.inner.space_id)
    }
}

impl<'a, E: Event<'a>> EventHeader<E> {
    #[inline]
    pub const fn new_core(time: u32, flags: EventFlags) -> Self
    where
        E: Event<'a, EventSpace = CoreEventSpace<'a>>,
    {
        Self::new_for_space(EventSpaceId::core(), time, flags)
    }

    #[inline]
    pub const fn new_for_space(
        space_id: EventSpaceId<E::EventSpace>,
        time: u32,
        flags: EventFlags,
    ) -> Self {
        Self {
            inner: clap_event_header {
                size: core::mem::size_of::<E>() as u32,
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
    pub fn new_with_flags(time: u32, flags: EventFlags) -> Self {
        Self::new_for_space(EventSpaceId::core(), time, flags)
    }

    #[inline]
    pub fn new(time: u32) -> Self {
        Self::new_with_flags(time, EventFlags::empty())
    }
}

use crate::events::spaces::CoreEventSpace;
use crate::events::spaces::EventSpaceId;
use crate::events::Event;
use clap_sys::events::{
    CLAP_EVENT_DONT_RECORD, CLAP_EVENT_IS_LIVE, CLAP_EVENT_PARAM_GESTURE_BEGIN,
    CLAP_EVENT_PARAM_GESTURE_END,
};

bitflags! {
    #[repr(C)]
    pub struct EventFlags: u32 {
        const IS_LIVE = CLAP_EVENT_IS_LIVE;
        const BEGIN_ADJUST = CLAP_EVENT_PARAM_GESTURE_BEGIN as u32;
        const END_ADJUST = CLAP_EVENT_PARAM_GESTURE_END as u32;
        const DONT_RECORD = CLAP_EVENT_DONT_RECORD;
    }
}

impl<E> PartialEq for EventHeader<E> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.time == other.inner.time
    }
}

impl<E> Eq for EventHeader<E> {}

impl<E> fmt::Debug for EventHeader<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EventHeader")
            .field("time", &self.time())
            .field("flags", &self.flags())
            .field("type_id", &self.inner.type_)
            .field("space_id", &self.inner.space_id)
            .field("payload_size", &self.payload_size())
            .finish()
    }
}

impl<E> PartialOrd for EventHeader<E> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for EventHeader<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.time.cmp(&other.inner.time)
    }
}
