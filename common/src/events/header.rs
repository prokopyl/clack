use crate::events::spaces::{CoreEventSpace, EventSpaceId};
use crate::events::Event;
use bitflags::bitflags;
use clap_sys::events::clap_event_header;
use clap_sys::events::{CLAP_EVENT_DONT_RECORD, CLAP_EVENT_IS_LIVE};
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
    /// Returns the size of this event, in bytes.
    ///
    /// This size includes the size of the event header itself.
    /// If you need only the size of the payload, see [`payload_size`](Self::payload_size).
    #[inline]
    pub const fn size(&self) -> u32 {
        self.inner.size
    }

    /// Returns the size of this event's payload, in bytes.
    ///
    /// This size does not include the size of the event header itself.
    /// If you need to also account for the size of the payload, see [`size`](Self::size).
    #[inline]
    pub const fn payload_size(&self) -> u32 {
        self.inner
            .size
            .saturating_sub(core::mem::size_of::<EventHeader>() as u32)
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
    pub fn set_time(&mut self, time: u32) {
        self.inner.time = time
    }

    #[inline]
    pub const fn with_time(mut self, time: u32) -> Self {
        self.inner.time = time;
        self
    }

    #[inline]
    pub const fn flags(&self) -> EventFlags {
        EventFlags::from_bits_truncate(self.inner.flags)
    }

    #[inline]
    pub fn set_flags(&mut self, flags: EventFlags) {
        self.inner.flags = flags.bits();
    }

    #[inline]
    pub const fn with_flags(mut self, flags: EventFlags) -> Self {
        self.inner.flags = flags.bits();
        self
    }

    // Raw stuff

    /// Gets a typed event header from a raw, untyped header, without performing any checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure the given header matches the given event type `E`.
    #[inline]
    pub const unsafe fn from_raw_unchecked(header: &clap_event_header) -> &Self {
        // SAFETY: EventHeader is repr(C) and ABI compatible
        &*(header as *const clap_event_header as *const Self)
    }

    /// Gets a typed event header from a mutable reference to a raw, untyped header,
    /// without performing any checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure the given header matches the given event type `E`.
    #[inline]
    pub unsafe fn from_raw_unchecked_mut(header: &mut clap_event_header) -> &mut Self {
        // SAFETY: EventHeader is repr(C) and ABI compatible
        &mut *(header as *mut clap_event_header as *mut Self)
    }

    #[inline]
    pub const fn as_raw(&self) -> &clap_event_header {
        &self.inner
    }

    #[inline]
    pub fn as_raw_mut(&mut self) -> &mut clap_event_header {
        &mut self.inner
    }

    #[inline]
    pub const fn into_raw(self) -> clap_event_header {
        self.inner
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

impl<E: Event> EventHeader<E> {
    #[inline]
    pub const fn new_core(time: u32, flags: EventFlags) -> Self
    where
        E: for<'a> Event<EventSpace<'a> = CoreEventSpace<'a>>,
    {
        Self::new_for_space(EventSpaceId::core(), time, flags)
    }

    #[inline]
    pub const fn new_for_space(
        space_id: EventSpaceId<E::EventSpace<'_>>,
        time: u32,
        flags: EventFlags,
    ) -> Self {
        Self {
            inner: clap_event_header {
                size: core::mem::size_of::<E>() as u32,
                time,
                space_id: space_id.id(),
                type_: E::TYPE_ID,
                flags: flags.bits(),
            },
            _event: PhantomData,
        }
    }

    #[inline]
    pub fn space_id(&self) -> EventSpaceId<E::EventSpace<'static>> {
        // SAFETY: the EventHeader type guarantees the space_id correctness
        unsafe { EventSpaceId::new_unchecked(self.inner.space_id) }
    }
}

impl<'a, E: Event<EventSpace<'a> = CoreEventSpace<'a>>> EventHeader<E> {
    #[inline]
    pub const fn new_with_flags(time: u32, flags: EventFlags) -> Self {
        Self::new_for_space(EventSpaceId::core(), time, flags)
    }

    #[inline]
    pub const fn new(time: u32) -> Self {
        Self::new_with_flags(time, EventFlags::empty())
    }
}

impl<'a, E: Event<EventSpace<'a> = CoreEventSpace<'a>>> Default for EventHeader<E> {
    #[inline]
    fn default() -> Self {
        Self::new(0)
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventFlags: u32 {
        const IS_LIVE = CLAP_EVENT_IS_LIVE;
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
