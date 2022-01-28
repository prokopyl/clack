use crate::events::list::implementation::{raw_input_events, InputEventBuffer};
use crate::events::UnknownEvent;
use clap_sys::events::clap_input_events;
use std::marker::PhantomData;
use std::ops::{Index, Range};

/// An ordered list of timestamped events.
///
/// `InputEvents`s are always ordered: an event at index `i` will always have a timestamp smaller than
/// or equal to the timestamp of the next event at index `i + 1`.
///
/// `InputEvents`s do not own the event data, they are only lightweight wrappers around a compatible
/// event buffer (i.e. [`InputEventBuffer`]), see [`InputEvents::from_buffer`].
///
/// Unlike [`Vec`s](std::vec::Vec) or slices, `EventList`s only support a couple of operations:
/// retrieving an event from its index ([`get`](InputEvents::get)), and appending a new event to the
/// list ([`append`](InputEvents::append)).
///
/// This type also implements a few extra features based on these operations for convenience,
/// such as [`Iterator`](core::iter::IntoIterator) or [`Extend`](core::iter::Extend).
///
/// # Example
///```
/// use clack_common::events::{EventList, Event, TimestampedEvent};
/// use clack_common::events::event_types::NoteEvent;
/// let mut buf = vec![];
/// let mut event_list = EventList::from_buffer(&mut buf);
///
/// assert!(event_list.is_empty());
///
/// let event = TimestampedEvent::new(0, Event::NoteOn(NoteEvent::new(0, 12, 0, 4.2)));
/// event_list.append(&event);
///
/// assert_eq!(1, event_list.len());
/// assert_eq!(event, event_list[0]);
///
/// assert_eq!(1, buf.len());
/// assert_eq!(event, buf[0]);
/// ```
#[repr(C)]
pub struct InputEvents<'a> {
    inner: clap_input_events,
    _lifetime: PhantomData<&'a clap_input_events>,
}

impl<'a> InputEvents<'a> {
    #[inline]
    pub unsafe fn from_raw(raw: &'a clap_input_events) -> &'a Self {
        &*(raw as *const _ as *const _)
    }

    #[inline]
    pub const fn from_buffer<I: InputEventBuffer>(buffer: &'a I) -> Self {
        Self {
            inner: raw_input_events(buffer),
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { (self.inner.size)(self.inner.ctx) }
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
    type Item = &'a UnknownEvent;
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

const INDEX_ERROR: &str = "Indexed EventList out of bounds";

impl<'a> Index<usize> for InputEvents<'a> {
    type Output = UnknownEvent;

    #[inline]
    fn index(&self, index: usize) -> &'a Self::Output {
        self.get(index).expect(INDEX_ERROR)
    }
}

impl<'a> Extend<&'a UnknownEvent> for InputEvents<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'a UnknownEvent>>(&mut self, iter: T) {
        for event in iter {
            self.append(&event)
        }
    }
}

/// Immutable [`EventList`] iterator.
pub struct InputEventsIter<'a> {
    list: &'a InputEvents<'a>,
    range: Range<usize>,
}

impl<'a, 'list> Iterator for InputEventsIter<'a> {
    type Item = &'a UnknownEvent;

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
