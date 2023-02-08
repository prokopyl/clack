//! Audio-processing events and related utilities.
//!
//! Events notify a plugin's Audio Processor of anything that may change its audio output, such as
//! note on/off events, parameter changes, MIDI events, and more.
//!
//! All events in CLAP are sample-accurate time-stamped events ([`Event`]).
//! They are provided to the plugin's audio processor alongside the audio buffers through
//! [`InputEventBuffer`](io::InputEventBuffer)s and read from
//! [`OutputEventBuffer`](io::OutputEventBuffer)s
//! (see the plugin's `process` method).

use crate::events::spaces::*;
use clap_sys::events::clap_event_header;
use std::marker::PhantomData;

pub mod event_types;
pub mod io;
pub mod spaces;

mod header;
pub use header::*;

/// A specific event type.
///
/// # Safety
///
/// This trait allows casting to and from pointers to raw event headers. This means implementers of
/// this trait must enforce the following:
///
/// * The [`EventSpace`](Event::EventSpace) type *must* be the [`EventSpace`] implementation that
/// * [`TYPE_ID`](Event::TYPE_ID) *must* match the event ID from its
pub unsafe trait Event<'a>: Sized + 'a {
    const TYPE_ID: u16;
    type EventSpace: EventSpace<'a>;

    #[inline]
    fn raw_header(&self) -> &clap_event_header {
        unsafe { &*(self as *const Self as *const _) }
    }

    #[inline]
    fn header(&self) -> &EventHeader<Self> {
        unsafe { EventHeader::from_raw_unchecked(self.raw_header()) }
    }

    #[inline]
    fn as_unknown(&self) -> &UnknownEvent<'a> {
        unsafe { UnknownEvent::from_raw(self.raw_header()) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct UnknownEvent<'a> {
    header: EventHeader,
    _sysex_lifetime: PhantomData<&'a u8>,
}

impl<'a> UnknownEvent<'a> {
    /// Gets an unknown event from a raw event header.
    ///
    /// # Safety
    /// The caller *must* ensure that not only the contents of the header are valid, but also that
    /// they are immediately preceding the rest of the event struct matching the event and space IDs
    /// in the header.
    #[inline]
    pub const unsafe fn from_raw(header: &clap_event_header) -> &Self {
        // SAFETY: EventHeader is repr(C) and ABI compatible
        &*(header as *const _ as *const _)
    }

    #[inline]
    pub const fn as_raw(&self) -> &clap_event_header {
        self.header.as_raw()
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
        let raw = self.header.as_raw();
        if raw.space_id != space_id.id()
            || raw.type_ != E::TYPE_ID
            || raw.size != core::mem::size_of::<E>() as u32
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

    /// Casts this event as an event of a given type, without performing any checks.
    ///
    /// # Safety
    /// The caller *must* ensure the event is of the given type, otherwise this will perform an
    /// incorrect cast, leading to Undefined Behavior.
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
        if space_id.id() != self.header.space_id()?.id() {
            return None;
        }

        unsafe { S::from_unknown(self) }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: any data can be safely transmuted to a slice of bytes. This type also ensures
        // the size field is correct
        unsafe {
            core::slice::from_raw_parts(self as *const _ as *const _, self.header.size() as usize)
        }
    }

    /// Retrieves an event from a byte buffer, without performing any checks.
    ///
    /// # Safety
    ///
    /// The caller must ensure the byte buffer is properly aligned, and that is also contains a
    /// valid event header as well as the remaining of the event struct.
    #[inline]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &UnknownEvent {
        &*(bytes.as_ptr() as *const _)
    }
}

impl<'a, E: Event<'a>> PartialEq<E> for UnknownEvent<'a>
where
    E: PartialEq,
{
    fn eq(&self, other: &E) -> bool {
        match self.as_event_for_space::<E>(other.header().space_id()) {
            None => false,
            Some(s) => s.eq(other),
        }
    }
}
