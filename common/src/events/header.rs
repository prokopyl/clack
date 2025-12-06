//#![deny(missing_docs)]

use crate::events::Event;
use crate::events::spaces::{CoreEventSpace, EventSpaceId};
use bitflags::bitflags;
use clap_sys::events::clap_event_header;
use clap_sys::events::{CLAP_EVENT_DONT_RECORD, CLAP_EVENT_IS_LIVE};
use std::cmp::Ordering;
use std::fmt;
use std::marker::PhantomData;

/// The common metadata header of all CLAP events.
///
/// All CLAP events have a common header that contains various metadata about them.
///
/// Most of that metadata is information used to discover the event's type: [`type_id`],
/// [`space_id`] and [`size`]. This is used internally by the [`UnknownEvent`] type to perform
/// downcasting to concrete types.
///
/// All events also carry a sample-accurate timestamp (accessible through [`time`]), as well as
/// some additional [event flags] (through [`flags`]).
///
/// Because event headers internally contain type information for the event, the [`EventHeader`]
/// is generic over a specific event type parameter `E` in order to guarantee that [`type_id`],
/// [`space_id`] and [`size`] are coherent with the event's type.
///
/// The generic parameter `E` can either be a concrete [`Event`] type (as returned by the
/// [`Event::header`] method), or `()` for headers of an undetermined event type (as returned by the
/// [`UnknownEvent::header`] method).
///
/// [`type_id`]: EventHeader::type_id
/// [`space_id`]: EventHeader::space_id
/// [`size`]: EventHeader::size
/// [`time`]: EventHeader::time
/// [`flags`]: EventHeader::flags
/// [`UnknownEvent`]: super::UnknownEvent
/// [`UnknownEvent::header`]: super::UnknownEvent::header
/// [event flags]: EventFlags
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
            .saturating_sub(size_of::<EventHeader>() as u32)
    }

    /// The raw event type ID.
    #[inline]
    pub const fn type_id(&self) -> u16 {
        self.inner.type_
    }

    /// The timestamp at which this event occurs, in samples.
    ///
    /// This timestamp is relative to the frame count of the current `process` invocation.
    #[inline]
    pub const fn time(&self) -> u32 {
        self.inner.time
    }

    /// Sets the timestamp at which this event occurs, in samples.
    ///
    /// This timestamp is relative to the frame count of the current `process` invocation.
    #[inline]
    pub const fn set_time(&mut self, time: u32) {
        self.inner.time = time
    }

    /// Sets the timestamp at which this event occurs, in samples.
    ///
    /// This timestamp is relative to the frame count of the current `process` invocation.
    ///
    /// This method takes and returns ownership of the event header, allowing it to be used in a
    /// builder-style pattern.
    #[inline]
    pub const fn with_time(mut self, time: u32) -> Self {
        self.inner.time = time;
        self
    }

    /// The event's [flags](EventFlags).
    #[inline]
    pub const fn flags(&self) -> EventFlags {
        EventFlags::from_bits_truncate(self.inner.flags)
    }

    /// Sets the event's [flags](EventFlags).
    #[inline]
    pub const fn set_flags(&mut self, flags: EventFlags) {
        self.inner.flags = flags.bits();
    }

    /// Sets the event's [flags](EventFlags).
    ///
    /// This method takes and returns ownership of the event header, allowing it to be used in a
    /// builder-style pattern.
    #[inline]
    pub const fn with_flags(mut self, flags: EventFlags) -> Self {
        self.inner.flags = flags.bits();
        self
    }

    // Raw stuff

    /// Gets a shared reference typed event header from a mutable shared to a raw,
    /// C-FFI compatible header struct, without performing any checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure the given header matches the given event type `E`.
    #[inline]
    pub const unsafe fn from_raw_unchecked(header: &clap_event_header) -> &Self {
        // SAFETY: EventHeader is repr(C) and ABI compatible
        &*(header as *const clap_event_header as *const Self)
    }

    /// Gets a mutable reference typed event header from a mutable reference to a raw,
    /// C-FFI compatible header struct, without performing any checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure the given header matches the given event type `E`.
    #[inline]
    pub const unsafe fn from_raw_unchecked_mut(header: &mut clap_event_header) -> &mut Self {
        // SAFETY: EventHeader is repr(C) and ABI compatible
        &mut *(header as *mut clap_event_header as *mut Self)
    }

    /// Returns this header as a shared reference to a raw, C-FFI compatible header struct.
    #[inline]
    pub const fn as_raw(&self) -> &clap_event_header {
        &self.inner
    }

    /// Returns this header as a mutable reference to a raw, C-FFI compatible header struct.
    #[inline]
    pub const fn as_raw_mut(&mut self) -> &mut clap_event_header {
        &mut self.inner
    }

    /// Returns this header as a raw, C-FFI compatible header struct.
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

    /// The untyped event space ID, from this untyped event header.
    #[inline]
    pub const fn space_id(&self) -> Option<EventSpaceId> {
        EventSpaceId::new(self.inner.space_id)
    }
}

impl<E: Event> EventHeader<E> {
    /// Creates a new header for a specific core event type, containing the given `time` and `flags`.
    ///
    /// The [`type_id`](Self::type_id), [`space_id`](Self::space_id) and [`size`](Self::size)
    /// members are derived from the `E` type parameter.
    ///
    /// See the [`event_types`](super::event_types) module for a list of all the supported core
    /// event types.
    #[inline]
    pub const fn new_core(time: u32, flags: EventFlags) -> Self
    where
        E: for<'a> Event<EventSpace<'a> = CoreEventSpace<'a>>,
    {
        Self::new_for_space(EventSpaceId::core(), time, flags)
    }

    /// Creates a new header for a specific event type, of a given custom event space,
    /// containing the given `time` and `flags`.
    ///
    /// The [`type_id`](Self::type_id) and [`size`](Self::size) members are derived from the `E`
    /// type parameter.
    #[inline]
    pub const fn new_for_space(
        space_id: EventSpaceId<E::EventSpace<'_>>,
        time: u32,
        flags: EventFlags,
    ) -> Self {
        Self {
            inner: clap_event_header {
                size: size_of::<E>() as u32,
                time,
                space_id: space_id.id(),
                type_: E::TYPE_ID,
                flags: flags.bits(),
            },
            _event: PhantomData,
        }
    }

    /// The typed event space ID from this event header.
    #[inline]
    pub const fn space_id(&self) -> EventSpaceId<E::EventSpace<'static>> {
        // SAFETY: the EventHeader type guarantees the space_id correctness
        unsafe { EventSpaceId::new_unchecked(self.inner.space_id) }
    }
}

impl<E: for<'a> Event<EventSpace<'a> = CoreEventSpace<'a>>> Default for EventHeader<E> {
    #[inline]
    fn default() -> Self {
        Self::new_core(0, EventFlags::empty())
    }
}

bitflags! {
    /// Flags for a CLAP event.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventFlags: u32 {
        /// Indicates a live user event, for example a user turning a physical
        /// knob or playing a physical key.
        const IS_LIVE = CLAP_EVENT_IS_LIVE;

        /// Indicates that the event should not be recorded.
        ///
        /// For example this is useful when a parameter changes because of a
        /// MIDI CC, because if the host records both the MIDI CC automation and
        /// the parameter automation there will be a conflict.
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
