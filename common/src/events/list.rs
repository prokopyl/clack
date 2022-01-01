use crate::events::TimestampedEvent;
use clap_sys::events::{clap_event, clap_event_list};
use std::marker::PhantomData;
use std::ops::{Index, Range};

mod implementation;

pub use implementation::EventBuffer;

/// An ordered list of timestamped events.
///
/// `EventList`s are always ordered: an event at index `i` will always have a timestamp smaller than
/// or equal to the timestamp of the next event at index `i + 1`.
///
/// `EventList`s do not own the event data, they are only lightweight wrappers around a compatible
/// event buffer (i.e. [`EventBuffer`]), see [`EventList::from_buffer`].
///
/// Unlike [`Vec`s](std::vec::Vec) or slices, `EventList`s only support a couple of operations:
/// retrieving an event from its index ([`get`](EventList::get)), and appending a new event to the
/// list ([`append`](EventList::append)).
///
/// This type also implements a few extra features based on these operations for convenience,
/// such as [`Iterator`](core::iter::IntoIterator) or [`Extend`](core::iter::Extend).
///
/// # Example
///```
/// use clack_common::events::{EventList, EventType, TimestampedEvent};
/// use clack_common::events::event_types::NoteEvent;
/// let mut buf = vec![];
/// let mut event_list = EventList::from_buffer(&mut buf);
///
/// assert!(event_list.is_empty());
///
/// let event = TimestampedEvent::new(0, EventType::NoteOn(NoteEvent::new(0, 12, 0, 4.2)));
/// event_list.append(&event);
///
/// assert_eq!(1, event_list.len());
/// assert_eq!(event, event_list[0]);
///
/// assert_eq!(1, buf.len());
/// assert_eq!(event, buf[0]);
/// ```
#[repr(C)]
pub struct EventList<'a> {
    list: clap_event_list,
    _lifetime: PhantomData<&'a mut clap_event_list>,
}

impl<'a> EventList<'a> {
    /// Returns the number of events in the list.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { (self.list.size)(&self.list) as usize }
    }

    /// Returns if there are no events in the list.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Attempts to retrieve an event from the list using its `index`.
    ///
    /// If `index` is greater than or equal to [`len`](EventList::len), `None` is returned.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&'a TimestampedEvent<'a>> {
        unsafe {
            (self.list.get)(&self.list, index as u32)
                .as_ref()
                .map(TimestampedEvent::from_raw)
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
    pub fn append(&mut self, event: &TimestampedEvent) {
        unsafe { (self.list.push_back)(&self.list, event.as_raw()) }
    }

    /// Returns an iterator over the event list.
    #[inline]
    pub fn iter(&self) -> EventListIter {
        EventListIter {
            list: self,
            range: 0..self.len(),
        }
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
    /// allocations on the main thread.
    ///
    /// # Example
    /// ```
    /// use clack_common::events::{EventList, EventType, TimestampedEvent};
    /// use clack_common::events::event_types::NoteEvent;
    ///
    /// let event = TimestampedEvent::new(0, EventType::NoteOn(NoteEvent::new(0, 12, 0, 4.2)));
    /// let mut buf = vec![event];
    ///
    /// let event_list = EventList::from_buffer(&mut buf);
    /// assert_eq!(1, event_list.len());
    /// assert_eq!(event, event_list[0]);
    /// ```
    #[inline]
    pub fn from_buffer<'b: 'a, E: EventBuffer<'b>>(buffer: &'a mut E) -> Self {
        Self {
            _lifetime: PhantomData,
            list: clap_event_list {
                ctx: buffer as *mut _ as *mut _,
                size: size::<E>,
                get: get::<E>,
                push_back: push_back::<E>,
            },
        }
    }

    /// Creates a shared reference to an EventList from a given C FFI-compatible pointer.
    ///
    /// # Safety
    /// The caller must ensure the given pointer is valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_raw(raw: *const clap_event_list) -> &'a Self {
        // SAFETY: EventList has the same layout and is repr(C)
        &*(raw as *const _)
    }

    /// Creates a mutable reference to an EventList from a given C FFI-compatible pointer.
    ///
    /// # Safety
    /// The caller must ensure the given pointer is valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_raw_mut(raw: *const clap_event_list) -> &'a mut Self {
        // SAFETY: EventList has the same layout and is repr(C)
        &mut *(raw as *const _ as *mut _)
    }

    /// Returns C FFI-compatible pointer to this event list.
    ///
    /// This pointer is only valid until the list is dropped.
    #[inline]
    pub fn as_raw(&self) -> *const clap_event_list {
        &self.list
    }

    /// Returns C FFI-compatible mutable pointer to this event list.
    ///
    /// This pointer is only valid until the list is dropped.
    #[inline]
    pub fn as_raw_mut(&mut self) -> *mut clap_event_list {
        &mut self.list
    }
}

const INDEX_ERROR: &str = "Indexed EventList out of bounds";

impl<'a> Index<usize> for EventList<'a>
where
    EventList<'a>: 'a,
{
    type Output = TimestampedEvent<'a>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect(INDEX_ERROR)
    }
}

impl<'a> Extend<TimestampedEvent<'a>> for EventList<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = TimestampedEvent<'a>>>(&mut self, iter: T) {
        for event in iter {
            self.append(&event)
        }
    }
}

impl<'a: 'e, 'e> Extend<&'e TimestampedEvent<'a>> for EventList<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'e TimestampedEvent<'a>>>(&mut self, iter: T) {
        for event in iter {
            self.append(event)
        }
    }
}

impl<'a> IntoIterator for &'a EventList<'a> {
    type Item = &'a TimestampedEvent<'a>;
    type IntoIter = EventListIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, I: EventBuffer<'a>> From<&'a mut I> for EventList<'a> {
    #[inline]
    fn from(implementation: &'a mut I) -> Self {
        Self::from_buffer(implementation)
    }
}

/// Immutable [`EventList`] iterator.
pub struct EventListIter<'a> {
    list: &'a EventList<'a>,
    range: Range<usize>,
}

impl<'a, 'list> Iterator for EventListIter<'a> {
    type Item = &'a TimestampedEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().and_then(|i| self.list.get(i))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<'a> ExactSizeIterator for EventListIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<'a> DoubleEndedIterator for EventListIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().and_then(|i| self.list.get(i))
    }
}

unsafe extern "C" fn size<'a, E: EventBuffer<'a>>(list: *const clap_event_list) -> u32 {
    E::size(&*((*list).ctx as *const E)) as u32
}

unsafe extern "C" fn get<'a, E: EventBuffer<'a>>(
    list: *const clap_event_list,
    index: u32,
) -> *const clap_event {
    E::get(&*((*list).ctx as *const _), index as usize)
        .map(|e| e.as_raw() as *const _)
        .unwrap_or_else(::core::ptr::null)
}

unsafe extern "C" fn push_back<'a, E: EventBuffer<'a>>(
    list: *const clap_event_list,
    event: *const clap_event,
) {
    E::push_back(
        &mut *((*list).ctx as *const _ as *mut E),
        TimestampedEvent::from_raw(&*event),
    )
}
