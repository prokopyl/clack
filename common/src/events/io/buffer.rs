use crate::events::event_types::TransportEvent;
use crate::events::io::implementation::{InputEventBuffer, OutputEventBuffer};
use crate::events::io::{InputEvents, OutputEvents, TryPushError};
use crate::events::UnknownEvent;
use clap_sys::events::clap_event_header;
use core::mem::{size_of_val, MaybeUninit};
use std::ops::{Index, Range};

#[repr(C, align(8))]
#[derive(Copy, Clone)]
struct AlignedEventHeader(clap_event_header);

/// A buffer for ordered storing of heterogeneous [`UnknownEvent`]s.
///
/// This type is useful for dynamic storage of arbitrary events, as [`UnknownEvent`]s are
/// dynamically-sized types (DSTs) and can not be simply stored in e.g. a [`Vec`].
///
/// This type is also useful as the backing storage for plugin events, as it implements both the
/// [`InputEventBuffer`] and the [`OutputEventBuffer`] traits.
///
/// # Example
///
/// ```
/// use clack_common::events::event_types::ParamGestureBeginEvent;
/// use clack_common::events::EventHeader;
/// use clack_common::events::io::EventBuffer;
///
/// let mut buffer = EventBuffer::new();
/// assert!(buffer.is_empty());
///
/// let some_event = ParamGestureBeginEvent::new(EventHeader::new(6), 2);
/// buffer.push(&some_event);
/// assert_eq!(1, buffer.len());
/// assert_eq!(&buffer[0], &some_event);
/// ```
///
/// # Realtime Safety
///
/// This type is backed by `Vec` internally, and therefore holds similar realtime properties.
/// Reading from it is always realtime-safe, but pushing new events to it may not be.
///
/// To avoid allocations, hosts should use the [`with_capacity`](EventBuffer::with_capacity)
/// to pre-allocate a reasonable amount of space for plugins to send their events.
///
/// However, this is always a best-effort, and not a guarantee.
pub struct EventBuffer {
    headers: Vec<MaybeUninit<AlignedEventHeader>>, // force 64-bit alignment
    indexes: Vec<u32>,
}

#[inline]
pub(crate) fn byte_index_to_value_index<T>(size: usize) -> usize {
    let type_size = core::mem::size_of::<T>();
    if type_size == 0 {
        0
    } else {
        size / type_size + if size % type_size > 0 { 1 } else { 0 }
    }
}

impl EventBuffer {
    /// Creates a new, empty [`EventBuffer`].
    #[inline]
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            indexes: Vec::new(),
        }
    }

    /// Creates a new empty [`EventBuffer`], but with enough pre-allocated capacity for the given
    /// number of standard events.
    ///
    /// Because CLAP events can have any arbitrary size, this method can only pre-allocate on a
    /// best-effort basis, only reserving enough space for the largest standard CLAP events.
    ///
    /// # Realtime Safety
    ///
    /// This method always allocates and is not realtime-safe, unless `events` is zero.
    #[inline]
    pub fn with_capacity(events: usize) -> Self {
        Self {
            // TransportEvent is the largest standard CLAP event.
            headers: Vec::with_capacity(events * core::mem::size_of::<TransportEvent>()),
            indexes: Vec::with_capacity(events),
        }
    }

    /// Clears the buffer, removing all events.
    ///
    /// Note that this has no effect on the allocated capacity of the buffer.
    pub fn clear(&mut self) {
        self.indexes.clear();
        self.headers.clear();
    }

    /// Returns the number of events in this buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    /// Returns `true` if this buffer has no events in it (i.e. if `len == 0`), `false` otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indexes.is_empty()
    }

    /// Returns the event located at the given `index`.
    ///
    /// If `index` is out of bounds, this returns [`None`].
    #[inline]
    pub fn get(&self, index: u32) -> Option<&UnknownEvent<'static>> {
        <Self as InputEventBuffer>::get(self, index)
    }

    /// Returns an iterator of all the events contained in this buffer.
    #[inline]
    pub fn iter(&self) -> EventBufferIter {
        EventBufferIter {
            buffer: self,
            range: 0..self.len(),
        }
    }

    /// Sorts the events contained in this buffer, based on their time.
    ///
    /// It is necessary to sort the events before passing them to a plugin.
    pub fn sort(&mut self) {
        self.indexes.sort_by_key(|i| {
            // SAFETY: Registered indexes always have actual event headers written by append_header_data
            // PANIC: We used registered indexes, this should never panic
            let event = unsafe { self.headers[*i as usize].assume_init_ref() };
            event.0.time
        })
    }

    /// Inserts a given `event` at the given `position`, shifting all events after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `position > len`.
    pub fn insert<E: AsRef<UnknownEvent<'static>> + ?Sized>(&mut self, event: &E, position: usize) {
        let index = self.append_header_data(event.as_ref());
        self.indexes.insert(position, index as u32);
    }

    /// Pushes all events produced by the given `events` iterator at the end of the buffer.
    pub fn push_all<'a, E: AsRef<UnknownEvent<'static>> + ?Sized + 'a>(
        &mut self,
        events: impl IntoIterator<Item = &'a E>,
    ) {
        for e in events {
            self.push(e);
        }
    }

    /// Pushes the given event into the buffer.
    ///
    /// The event is always added at the end of the buffer.
    pub fn push<E: AsRef<UnknownEvent<'static>> + ?Sized>(&mut self, event: &E) {
        let index = self.append_header_data(event.as_ref());
        self.indexes.push(index as u32);
    }

    /// Produces an [`InputEvents`] that wraps this buffer as an [`InputEventBuffer`] implementation.
    ///
    /// This helper method is strictly equivalent to using [`InputEvents::from_buffer`].
    #[inline]
    pub fn as_input(&self) -> InputEvents {
        InputEvents::from_buffer(self)
    }

    /// Produces an [`OutputEvents`] that wraps this buffer as an [`OutputEventBuffer`] implementation.
    ///
    /// This helper method is strictly equivalent to using [`OutputEvents::from_buffer`].
    #[inline]
    pub fn as_output(&mut self) -> OutputEvents {
        OutputEvents::from_buffer(self)
    }

    fn append_header_data(&mut self, event: &UnknownEvent<'static>) -> usize {
        let index = self.headers.len();
        let event_bytes = event.as_bytes();
        let bytes = self.allocate_mut(event_bytes.len());

        // PANIC: bytes is guaranteed by allocate_mut to be just the right size
        bytes.copy_from_slice(event_bytes);
        index
    }

    fn allocate_mut(&mut self, byte_size: usize) -> &mut [u8] {
        let previous_len = self.headers.len();
        let headers_size = byte_index_to_value_index::<AlignedEventHeader>(byte_size);
        self.headers
            .resize(previous_len + headers_size, MaybeUninit::zeroed());

        // PANIC: we just resized, this should not panic unless there is a bug in the implementation
        let new_elements = &mut self.headers[previous_len..];

        // SAFETY: casting anything to bytes is always safe
        let new_bytes = unsafe {
            core::slice::from_raw_parts_mut(
                new_elements.as_mut_ptr() as *mut u8,
                size_of_val(new_elements),
            )
        };

        // PANIC: this should not panic unless there is a bug in the implementation
        &mut new_bytes[..byte_size]
    }
}

impl<'a> IntoIterator for &'a EventBuffer {
    type Item = &'a UnknownEvent<'a>;
    type IntoIter = EventBufferIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl InputEventBuffer<'static> for EventBuffer {
    #[inline]
    fn len(&self) -> u32 {
        self.indexes.len() as u32
    }

    fn get(&self, index: u32) -> Option<&UnknownEvent<'static>> {
        let header_index = (*self.indexes.get(index as usize)?) as usize;
        // SAFETY: Registered indexes always have actual event headers written by append_header_data
        // PANIC: We used registered indexes, this should never panic
        let event = unsafe { self.headers[header_index].assume_init_ref() };
        let event_size = event.0.size as usize;
        let header_end_index =
            byte_index_to_value_index::<AlignedEventHeader>(event_size) + header_index;

        let event = &self.headers[header_index..header_end_index];

        // SAFETY: we know the [0..event_size] slice is initialized
        let event_bytes =
            unsafe { core::slice::from_raw_parts(event.as_ptr() as *const u8, event_size) };

        // SAFETY: the event header was written from a valid UnknownEvent in append_header_data
        Some(unsafe { UnknownEvent::from_bytes_unchecked(event_bytes) })
    }
}

impl OutputEventBuffer<'static> for EventBuffer {
    fn try_push(&mut self, event: &UnknownEvent<'static>) -> Result<(), TryPushError> {
        self.push(event);

        Ok(())
    }
}

impl Default for EventBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

const INDEX_ERROR: &str = "Indexed EventBuffer out of bounds";

impl Index<usize> for EventBuffer {
    type Output = UnknownEvent<'static>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index as u32).expect(INDEX_ERROR)
    }
}

/// An iterator over the events contained in an [`EventBuffer`].
pub struct EventBufferIter<'a> {
    buffer: &'a EventBuffer,
    range: Range<usize>,
}

impl<'a> Iterator for EventBufferIter<'a> {
    type Item = &'a UnknownEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_index = self.range.next()?;
        self.buffer.get(next_index as u32)
    }
}

#[cfg(test)]
mod test {
    use crate::events::event_types::MidiEvent;
    use crate::events::io::EventBuffer;
    use crate::events::{Event, EventFlags, EventHeader};

    #[test]
    fn it_works() {
        let event_0 = MidiEvent::new(EventHeader::new_core(0, EventFlags::empty()), 0, [0; 3]);
        let event_1 = MidiEvent::new(EventHeader::new_core(1, EventFlags::empty()), 0, [1; 3]);
        let event_2 = MidiEvent::new(EventHeader::new_core(2, EventFlags::empty()), 0, [2; 3]);
        let event_3 = MidiEvent::new(EventHeader::new_core(3, EventFlags::empty()), 0, [3; 3]);

        let events_1 = [event_1, event_2];
        let events_2 = [event_0, event_3];

        let mut buffer = EventBuffer::new();
        buffer.push_all(
            [events_1, events_2]
                .iter()
                .flatten()
                .map(|e| e.as_unknown()),
        );

        buffer.sort();

        assert_eq!(Some(&event_0), buffer.get(0).unwrap().as_event());
        assert_eq!(Some(&event_1), buffer.get(1).unwrap().as_event());
        assert_eq!(Some(&event_2), buffer.get(2).unwrap().as_event());
        assert_eq!(Some(&event_3), buffer.get(3).unwrap().as_event());
    }
}
