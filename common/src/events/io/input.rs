use crate::events::io::implementation::{raw_input_events, InputEventBuffer};
use crate::events::UnknownEvent;
use clap_sys::events::clap_input_events;
use std::marker::PhantomData;
use std::ops::{Index, Range};

/// An input list of timestamped events.
///
/// `InputEvents`s are always ordered: an event at index `i` will always have a timestamp smaller than
/// or equal to the timestamp of the next event at index `i + 1`.
///
/// `InputEvents`s do not own the event data, they are only lightweight wrappers around a compatible
/// event buffer (i.e. [`InputEventBuffer`]), see [`InputEvents::from_buffer`] as the default implementation.
///
/// Unlike [`Vec`s](std::vec::Vec) or slices, `InputEvents`s only support retrieving an event from
/// its index ([`get`](InputEvents::get)). It also implements a few extra features for convenience,
/// such as [`Iterator`](core::iter::IntoIterator).
///
/// # Example
///```
/// # #[cfg(not(miri))] let _: () = { // TODO: MIRI does not support C-style inheritance casts
/// use clack_common::events::{Event, EventHeader};
/// use clack_common::events::event_types::{NoteEvent, NoteOnEvent};
/// use clack_common::events::io::{EventBuffer, InputEvents, OutputEventBuffer};
///
/// let mut buf = EventBuffer::new();
/// let event = NoteOnEvent(NoteEvent::new(EventHeader::new(0), 60, 0, 12, 0, 4.2));
/// buf.try_push(event.as_unknown());
/// assert_eq!(1, buf.len());
///
/// let mut input_events = InputEvents::from_buffer(&mut buf);
///
/// assert_eq!(1, input_events.len());
/// assert_eq!(&event, input_events[0].as_event().unwrap());
/// # };
/// ```
#[repr(C)]
pub struct InputEvents<'a> {
    inner: clap_input_events,
    _lifetime: PhantomData<&'a clap_input_events>,
}

impl<'a> InputEvents<'a> {
    /// Creates a shared reference to an InputEvents list from a given C FFI-compatible pointer.
    ///
    /// # Safety
    /// The caller must ensure the given pointer is valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_raw(raw: &'a clap_input_events) -> &'a Self {
        &*(raw as *const _ as *const _)
    }

    /// Returns a C FFI-compatible pointer to this event list.
    ///
    /// This pointer is only valid until the list is dropped.
    #[inline]
    pub fn as_raw(&self) -> &clap_input_events {
        &self.inner
    }

    #[inline]
    pub fn from_buffer<I: InputEventBuffer>(buffer: &'a I) -> Self {
        Self {
            inner: raw_input_events(buffer),
            _lifetime: PhantomData,
        }
    }

    /// Returns the number of events in the list.
    #[inline]
    pub fn len(&self) -> u32 {
        unsafe { (self.inner.size)(&self.inner) }
    }

    /// Returns if there are no events in the list.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Attempts to retrieve an event from the list using its `index`.
    ///
    /// If `index` is out of bounds, `None` is returned.
    #[inline]
    pub fn get(&self, index: u32) -> Option<&UnknownEvent<'a>> {
        unsafe {
            (self.inner.get)(&self.inner, index)
                .as_ref()
                .map(|e| UnknownEvent::from_raw(e))
        }
    }

    #[inline]
    pub fn iter(&self) -> InputEventsIter {
        InputEventsIter {
            list: self,
            range: 0..self.len(),
        }
    }
}

impl<'a> IntoIterator for &'a InputEvents<'a> {
    type Item = &'a UnknownEvent<'a>;
    type IntoIter = InputEventsIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, I: InputEventBuffer> From<&'a mut I> for InputEvents<'a> {
    #[inline]
    fn from(implementation: &'a mut I) -> Self {
        Self::from_buffer(implementation)
    }
}

const INDEX_ERROR: &str = "Indexed InputEvents list out of bounds";

impl<'a> Index<usize> for InputEvents<'a> {
    type Output = UnknownEvent<'a>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index as u32).expect(INDEX_ERROR)
    }
}

/// Immutable [`InputEvents`] iterator.
pub struct InputEventsIter<'a> {
    list: &'a InputEvents<'a>,
    range: Range<u32>,
}

impl<'a> Iterator for InputEventsIter<'a> {
    type Item = &'a UnknownEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().and_then(|i| self.list.get(i))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<'a> ExactSizeIterator for InputEventsIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<'a> DoubleEndedIterator for InputEventsIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().and_then(|i| self.list.get(i))
    }
}
