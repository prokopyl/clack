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
use std::fmt::{Debug, Formatter};

pub mod event_types;
pub mod io;
pub mod spaces;

mod header;
mod helpers;
mod pckn;

pub use header::*;
pub use pckn::*;

/// A specific event type.
///
/// # Safety
///
/// This trait allows casting to and from pointers to raw event headers. This means implementers of
/// this trait must enforce the following:
///
/// * The [`EventSpace`](Event::EventSpace) type *must* be the [`EventSpace`] implementation that
///   it belongs to;
/// * [`TYPE_ID`](Event::TYPE_ID) *must* match the event ID from its type.
/// * The type *must* be ABI-compatible with the matching raw, C-FFI compatible event type.
/// * All instances of this type *must* be initialized and valid.
pub unsafe trait Event: AsRef<UnknownEvent> + Sized + 'static {
    const TYPE_ID: u16;
    type EventSpace<'a>: EventSpace<'a>;

    #[inline]
    fn flags(&self) -> EventFlags {
        self.header().flags()
    }

    #[inline]
    fn set_flags(&mut self, flags: EventFlags) {
        self.header_mut().set_flags(flags)
    }

    #[inline]
    fn with_flags(mut self, flags: EventFlags) -> Self {
        self.header_mut().set_flags(flags);
        self
    }

    #[inline]
    fn time(&self) -> u32 {
        self.header().time()
    }

    #[inline]
    fn set_time(&mut self, time: u32) {
        self.header_mut().set_time(time)
    }

    #[inline]
    fn with_time(mut self, time: u32) -> Self {
        self.header_mut().set_time(time);
        self
    }

    #[inline]
    fn header(&self) -> &EventHeader<Self> {
        // SAFETY: this trait guarantees the raw_header points to an event
        // header that matches the current type.
        unsafe { EventHeader::from_raw_unchecked(self.raw_header()) }
    }

    #[inline]
    fn header_mut(&mut self) -> &mut EventHeader<Self> {
        // SAFETY: this trait guarantees the raw_header points to an event
        // header that matches the current type.
        unsafe { EventHeader::from_raw_unchecked_mut(self.raw_header_mut()) }
    }

    #[inline]
    fn as_unknown(&self) -> &UnknownEvent {
        // SAFETY: this trait guarantees the raw_header points to an initialized and valid event.
        unsafe { UnknownEvent::from_raw(self.raw_header()) }
    }

    #[inline]
    fn raw_header(&self) -> &clap_event_header {
        // SAFETY: This trait guarantees self points to an initialized and valid event.
        unsafe { &*(self as *const Self as *const clap_event_header) }
    }

    #[inline]
    fn raw_header_mut(&mut self) -> &mut clap_event_header {
        // SAFETY: This trait guarantees self points to an initialized and valid event.
        unsafe { &mut *(self as *mut Self as *mut clap_event_header) }
    }
}

/// An event of an undetermined type.
///
/// CLAP event types come in a variety of different sizes. Therefore, unknown events do not have
/// their sizes known at compile-time, making this type a
/// [DST](https://doc.rust-lang.org/reference/dynamically-sized-types.html) which can only be
/// operated on through references (`&`).
///
/// Moreover, because CLAP supports defining using custom events, it's possible for any host or
/// plugin to encounter an event type it cannot possibly figure out the concrete type of.
///
/// The [`UnknownEvent`] type allows to safely work with events without knowing their type.
/// Their headers can be retrieved with the [`header()`](UnknownEvent::header) method, and they can
/// be directly copied around through [`InputEventBuffer`s](io::InputEventBuffer) and
/// [`OutputEventBuffer`s](io::OutputEventBuffer).
///
/// To try and figure out an event's concrete type, the [`as_event()`](UnknownEvent::as_event)
/// method can be used to try and downcast it to a specific event type. The
/// [`as_core_event`](UnknownEvent::as_core_event) can also be used to check if the event is a
/// standard CLAP event, and then further `match`ed to find its specific type.
///
/// Alternatively, one can also use the [`as_event_for_space`](UnknownEvent::as_event_for_space) and
/// [`as_event_space`](UnknownEvent::as_event_space) methods, which are respectively identical but
/// for custom [event spaces](EventSpace), by checking against an ID from the `event_registry` extension.
#[repr(transparent)]
pub struct UnknownEvent {
    data: [u8],
}

impl UnknownEvent {
    /// Returns the header of this event.
    #[inline]
    pub const fn header(&self) -> &EventHeader {
        // SAFETY: Pointer is guaranteed to be valid from constructors
        unsafe { EventHeader::from_raw(&*self.as_raw()) }
    }

    /// Attempts to downcast this event to a specific standard CLAP event type.
    ///
    /// This returns a down-casted reference to the event if the event matches the given type, or
    /// `None` if it doesn't.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_common::events::event_types::NoteOnEvent;
    /// use clack_common::events::UnknownEvent;
    ///
    /// fn handle_event(event: &UnknownEvent) {
    ///     if let Some(note_on_event) = event.as_event::<NoteOnEvent>() {
    ///         // Handle the Note On event
    ///         # let _ = note_on_event;
    ///     } else {
    ///         // This is not a Note On event
    ///     }
    /// }
    ///
    /// ```
    #[inline]
    pub fn as_event<'s, E: Event<EventSpace<'s> = CoreEventSpace<'s>>>(&self) -> Option<&E> {
        self.as_event_for_space(EventSpaceId::core())
    }

    /// Attempts to downcast this event to the [core event space](CoreEventSpace), allowing it to be
    /// match against all the standard CLAP event types.
    ///
    /// This returns `None` instead if the given event doesn't match the known standard events.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_common::events::event_types::NoteOnEvent;
    /// use clack_common::events::UnknownEvent;
    ///
    /// fn handle_event(event: &UnknownEvent) {
    ///     use clack_common::events::spaces::CoreEventSpace::*;
    ///
    ///     let Some(core_event) = event.as_core_event() else {
    ///         // This is not a known standard event.
    ///         return;
    ///     };
    ///
    ///     match core_event {
    ///         NoteOn(e) => { /* Handle the Note On event */ }
    ///         NoteOff(e) => { /* Handle the Note Off event */ }
    ///         ParamValue(e) => { /* Handle the Parameter Value event */ }
    ///         _ => { /* This is another event type than the ones we chose to handle */ }
    ///     }
    /// }
    ///
    /// ```
    #[inline]
    pub fn as_core_event(&self) -> Option<CoreEventSpace<'_>> {
        self.as_event_space(EventSpaceId::core())
    }

    /// Attempts to downcast this event to a specific event type from a given [event space](EventSpace).
    ///
    /// This returns a down-casted reference to the event if the event matches the given type and
    /// space, or `None` if it doesn't.
    #[inline]
    pub fn as_event_for_space<E: Event>(
        &self,
        space_id: EventSpaceId<E::EventSpace<'_>>,
    ) -> Option<&E> {
        let raw = self.header().as_raw();
        if raw.space_id != space_id.id()
            || raw.type_ != E::TYPE_ID
            || raw.size != core::mem::size_of::<E>() as u32
        {
            return None;
        }

        // SAFETY: this type guarantees the header is followed by event data, and we just checked the space_id, type and size fields
        Some(unsafe { self.as_event_unchecked() })
    }

    /// Attempts to downcast this event to the given [event space](EventSpace), allowing it to be
    /// match against all the event types of that space.
    ///
    /// This returns `None` instead if the given event doesn't match the known events from that
    /// event space.
    #[inline]
    pub fn as_event_space<'s, S: EventSpace<'s>>(&'s self, space_id: EventSpaceId<S>) -> Option<S> {
        if space_id.id() != self.header().space_id()?.id() {
            return None;
        }

        // SAFETY: we just checked the event space ID is valid for the current event.
        unsafe { S::from_unknown(self) }
    }

    /// Casts this event as an event of a given type, without performing any checks.
    ///
    /// # Safety
    /// The caller *must* ensure the event is of the given type, otherwise this will perform an
    /// incorrect cast, leading to Undefined Behavior.
    #[inline]
    pub unsafe fn as_event_unchecked<E: Event>(&self) -> &E {
        &*(self as *const _ as *const E)
    }

    /// Gets an unknown event from a raw event header.
    ///
    /// # Safety
    /// The caller *must* ensure that not only the contents of the header are valid, but also that
    /// they are immediately preceding the rest of the event struct matching the event and space IDs
    /// in the header.
    #[inline]
    pub const unsafe fn from_raw<'e>(header: *const clap_event_header) -> &'e Self {
        // SAFETY: no need to check for len > 0, since the data slice includes the header itself.
        let data = core::slice::from_raw_parts(header as *const _, (*header).size as usize);
        // SAFETY: The caller guarantees the right number of bytes is available after the given pointer in the size field.
        Self::from_bytes_unchecked(data)
    }

    /// Returns a raw, C-FFI compatible event header pointer to this event.
    #[inline]
    pub const fn as_raw(&self) -> *const clap_event_header {
        self.data.as_ptr() as *const clap_event_header
    }

    /// Returns the event as a raw byte buffer. This includes the event's (header)[EventHeader].
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Retrieves an event from a byte buffer, without performing any checks.
    ///
    /// The given buffer must include the event's (header)[EventHeader].
    ///
    /// # Safety
    ///
    /// The caller must ensure the byte buffer is properly aligned, and that is also contains a
    /// valid event header as well as the remaining of the event struct.
    #[inline]
    pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &UnknownEvent {
        &*(bytes as *const [u8] as *const _)
    }
}

impl<E: Event> PartialEq<E> for UnknownEvent
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

impl Debug for UnknownEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.as_core_event() {
            Some(e) => Debug::fmt(&e, f),
            None => f
                .debug_struct("UnknownEvent")
                .field("header", self.header())
                .finish(),
        }
    }
}

impl AsRef<UnknownEvent> for UnknownEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self
    }
}

#[inline]
pub(crate) const fn ensure_event_matches<'s, E: Event<EventSpace<'s> = CoreEventSpace<'s>>>(
    header: &clap_event_header,
) {
    if header.space_id != 0 {
        panic_event_space_mismatch()
    }

    if header.type_ != E::TYPE_ID {
        panic_event_type_mismatch()
    }
}

#[inline(never)]
#[cold]
const fn panic_event_space_mismatch() {
    panic!(
        "CLAP Event mismatch: expected core event space (ID 0), got a non-core event space instead.",
    )
}

#[inline(never)]
#[cold]
const fn panic_event_type_mismatch() {
    panic!("CLAP Event mismatch: got a different event ID from expected")
}
