use crate::events::io::implementation::{raw_output_events, OutputEventBuffer};
use crate::events::UnknownEvent;
use clap_sys::events::clap_output_events;
use std::marker::PhantomData;

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
pub struct OutputEvents<'a> {
    inner: clap_output_events,
    _lifetime: PhantomData<&'a clap_output_events>,
}

impl<'a> OutputEvents<'a> {
    /// Creates a mutable reference to an EventList from a given C FFI-compatible pointer.
    ///
    /// # Safety
    /// The caller must ensure the given pointer is valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_raw_mut(raw: &mut clap_output_events) -> &'a mut Self {
        // SAFETY: EventList has the same layout and is repr(C)
        &mut *(raw as *mut _ as *mut _)
    }

    /// Returns a C FFI-compatible mutable pointer to this event list.
    ///
    /// This pointer is only valid until the list is dropped.
    #[inline]
    pub fn as_raw_mut(&mut self) -> &mut clap_output_events {
        unsafe { &mut *(self as *mut _ as *mut _) }
    }

    /// Create a new event list by wrapping an event buffer implementation.
    ///
    /// This buffer may have been used by a previous `EventList`, and will keep all events that
    /// have been [`append`ed](EventList::append) into it. This allows to reuse event buffers
    /// between process trips.
    ///
    /// # Realtime Safety
    ///
    /// This is a very cheap, realtime-safe operation that does not change the given buffer in any
    /// way. However, since users of the `EventList` may call `append` on it, the buffer should have
    /// a reasonable amount of storage pre-allocated, as to not have to perform additional
    /// allocations on the audio thread.
    ///
    /// # Example
    /// ```
    /// use clack_common::events::{EventList, Event, TimestampedEvent};
    /// use clack_common::events::event_types::NoteEvent;
    ///
    /// let event = TimestampedEvent::new(0, Event::NoteOn(NoteEvent::new(0, 12, 0, 4.2)));
    /// let mut buf = vec![event];
    ///
    /// let event_list = EventList::from_buffer(&mut buf);
    /// assert_eq!(1, event_list.len());
    /// assert_eq!(event, event_list[0]);
    /// ```
    #[inline]
    pub fn from_buffer<I: OutputEventBuffer>(buffer: &'a mut I) -> Self {
        Self {
            inner: raw_output_events(buffer),
            _lifetime: PhantomData,
        }
    }

    /// Appends a copy of the given event to the list.
    ///
    /// Note that the event is not guaranteed to be added at the end of the list: in order to
    /// efficiently preserve ordering, some implementations may choose to insert it in a position
    /// following all events with smaller timestamps, and preceding all events with greater
    /// timestamps.
    ///
    /// For best performance however, it is recommended to insert the events in order if possible.
    ///
    /// # Realtime Safety
    ///
    /// This operation may cause the underlying event buffer to be reallocated by the host, therefore
    /// realtime safety cannot be guaranteed. However, it is expected for hosts to pre-allocate a
    /// reasonable amount of space before forwarding the list to the plugin, in order to make
    /// allocations as unlikely as possible.
    #[inline]
    pub fn push_back(&mut self, event: &UnknownEvent) {
        unsafe { (self.inner.push_back.unwrap())(&self.inner, event.as_raw()) }
    }
}

impl<'a, I: OutputEventBuffer> From<&'a mut I> for OutputEvents<'a> {
    #[inline]
    fn from(implementation: &'a mut I) -> Self {
        Self::from_buffer(implementation)
    }
}

impl<'a: 'b, 'b> Extend<&'b UnknownEvent<'a>> for OutputEvents<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'b UnknownEvent<'a>>>(&mut self, iter: T) {
        for event in iter {
            self.push_back(event)
        }
    }
}
