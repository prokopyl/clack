use crate::events::io::implementation::{raw_input_events, InputEventBuffer};
use crate::events::io::EventBatcher;
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
/// Unlike [`Vec`s](Vec) or slices, `InputEvents` only support retrieving an event from
/// its index ([`get`](InputEvents::get)). It also implements a few extra features for convenience,
/// such as [`Iterator`](IntoIterator).
///
/// # Example
///```
/// use clack_common::events::{Event, EventHeader};
/// use clack_common::events::event_types::{NoteEvent, NoteOnEvent};
/// use clack_common::events::io::{EventBuffer, InputEvents};
///
/// let event = NoteOnEvent(NoteEvent::new(EventHeader::new(0), 60, 0, 12, 0, 4.2));
/// let buf = [event];
/// let mut input_events = InputEvents::from_buffer(&buf);
///
/// assert_eq!(1, input_events.len());
/// assert_eq!(&event, input_events[0].as_event().unwrap());
/// ```
#[repr(C)]
pub struct InputEvents<'a> {
    inner: clap_input_events,
    _lifetime: PhantomData<(&'a clap_input_events, *const ())>,
}

impl<'a> InputEvents<'a> {
    /// Creates a shared reference to an [`InputEvents`] list from a given C FFI-compatible pointer.
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

    /// Returns an [`InputEvents`] from a reference [`InputEventBuffer`] implementation.
    ///
    /// The most common [`InputEventBuffer`] implementors are the
    /// [`EventBuffer`](crate::events::io::EventBuffer), and arrays or slices of either any
    /// [`Event`](crate::events::Event) type, or of [`UnknownEvent`] references.
    ///
    /// See the [`InputEventBuffer`] implementation docs for a list of all types that can be used
    /// by default.
    #[inline]
    pub const fn from_buffer<'b: 'a, I: InputEventBuffer<'b>>(buffer: &'a I) -> Self {
        Self {
            inner: raw_input_events(buffer),
            _lifetime: PhantomData,
        }
    }

    /// Returns an [`InputEvents`] that is always empty.
    ///
    /// # Example
    ///
    /// ```
    /// use clack_common::events::io::InputEvents;
    ///
    /// let no_events = InputEvents::empty();
    /// assert_eq!(0, no_events.len());
    /// assert!(no_events.get(0).is_none());
    /// ```
    #[inline]
    pub const fn empty() -> InputEvents<'static> {
        InputEvents::from_buffer::<[&UnknownEvent<'static>; 0]>(&[])
    }

    /// Returns the number of events in the list.
    #[inline]
    pub fn len(&self) -> u32 {
        match self.inner.size {
            None => 0,
            // SAFETY: this function pointer is safely initialized by from_raw or from_buffer
            Some(s) => unsafe { s(&self.inner) },
        }
    }

    /// Returns `true` if there are no events in the list.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Attempts to retrieve an event from the list using its `index`.
    ///
    /// If no event is found at the given `index`, `None` is returned.
    ///
    /// Implementors of this method *SHOULD* always return an event if the `index` is in bounds
    /// (`0..len`), and *SOULD NOT* return one if the `index` is out of bounds. However, this isn't
    /// a guarantee.
    ///
    /// Because incorrect implementations can be found in the wild, users of this method should
    /// always check the returned `Option`, and skip if `None` is returned while
    /// getting an expected event using an index that is in bounds.
    ///
    /// This also means that users should *not* rely on this method returning `None` to stop an
    /// iteration, or `Some` to continue it. Always rely on the value of [`len`](InputEvents::len)
    /// first if you wish to iterate manually.
    ///
    /// # Panics
    ///
    /// While this method itself does not panic, some hosts (e.g. Bitwig) may elect to crash the
    /// the plugin if the given `index` is out of bounds.
    ///
    /// Therefore, users should be careful while calling this method. Always check the value of
    /// [`len`](InputEvents::len) before attempting to call this method, to ensure that `index` is
    /// always in bounds.
    ///
    /// # See also
    ///
    /// Use the [`iter`](InputEvents::iter) method to iterate on all input events, which handles
    /// all of the above edge-cases.
    #[inline]
    pub fn get(&self, index: u32) -> Option<&UnknownEvent<'a>> {
        // SAFETY: this function pointer is safely initialized by from_raw or from_buffer
        let event = unsafe { self.inner.get?(&self.inner, index) };
        if event.is_null() {
            return None;
        };

        // SAFETY: the returned event pointer is guaranteed to be valid by from_raw or from_buffer
        unsafe { Some(UnknownEvent::from_raw(event)) }
    }

    /// Returns an iterator over all the events in this [`InputEvents`].
    #[inline]
    pub fn iter(&self) -> InputEventsIter {
        InputEventsIter {
            list: self,
            range: 0..self.len(),
        }
    }

    /// Returns an iterator over a specific sub-range of the events in this [`InputEvents`].
    ///
    /// If the given `range` is out of bounds, then `None` is returned.
    #[inline]
    pub fn iter_range(&self, range: Range<u32>) -> Option<InputEventsIter> {
        let len = self.len();
        if range.start > len || range.end > len {
            None
        } else {
            Some(InputEventsIter { list: self, range })
        }
    }

    /// Returns an iterator that batches all events for easier processing alongside sample buffers.
    ///
    /// This iterator will split the stream of events into multiple
    /// [`EventBatch`es](crate::events::io::EventBatch), so that:
    /// * Each batch will only have events on its very first sample.
    /// * All events that happen on the same sample are batched together.
    ///
    /// This makes batch processing of audio samples easier, while still being able to handle
    /// incoming events that may happen on each sample.
    ///
    /// See the [`EventBatch`](crate::events::io::EventBatch) documentation for more information.
    ///
    /// # Visual example
    ///
    /// In this example, a plugin will receive a buffer of audio samples (represented with `.`s),
    /// alongside multiple input events (represented by `E`s), which happen at precise,
    /// sample-accurate times.
    ///
    /// The timeline of a processing chunk might look like something like this:
    ///
    ///```text
    ///
    ///         E           E         E         E         
    ///                     E                   E         
    ///                                         E         
    /// . . . . . . . . . . . . . . . . . . . . . . . . .
    /// ```
    ///
    /// While some events are happening at the same time, there are some parts of the sample buffer
    /// where no event is occurring, which could be processed efficiently all at once.
    ///
    /// The [`EventBatcher`] finds these, and splits the stream into multiple
    /// [`EventBatch`es](crate::events::io::EventBatch) where events only happen at the beginning of
    /// each batch.
    ///
    /// ```text
    ///|       |E          |E        |E        |E        |
    ///|       |           |E        |         |E        |
    ///|       |           |         |         |E        |
    ///|. . . .|. . . . . .|. . . . .|. . . . .|. . . . .|
    ///| batch |   batch   |  batch  |  batch  |  batch  |
    /// ```
    ///
    /// # Example
    ///
    /// This example shows how to efficiently process the event batches alongside an audio buffer.
    ///
    /// ```
    /// use clack_common::events::io::InputEvents;
    /// # use clack_common::events::UnknownEvent;
    ///
    /// # fn batch_process(input_events: InputEvents, audio_buffer: &[f32]) {
    /// let input_events: InputEvents = /* ... */
    /// # input_events;
    /// let audio_buffer: &[f32] = /* ... */
    /// # audio_buffer;
    ///
    /// for event_bach in input_events.batch() {
    ///     // First, handle all events that could affect audio processing.
    ///     // (e.g. note on/off, parameter changes, etc.)
    ///    for event in event_bach.events() {
    ///        // (Handle the event...)
    ///    }
    ///
    ///    // Now, we can process the whole audio batch
    ///    let audio_batch: &[f32] = &audio_buffer[event_bach.sample_bounds()];
    ///    // (Process the audio samples...)
    /// }
    /// # }
    /// # let events: [&UnknownEvent<'static>; 0] = []; batch_process(InputEvents::from_buffer(&events), &[]);
    /// ```
    ///
    ///
    #[inline]
    pub fn batch(&self) -> EventBatcher {
        EventBatcher::new(self)
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

impl<'a, I: InputEventBuffer<'a>> From<&'a I> for InputEvents<'a> {
    #[inline]
    fn from(implementation: &'a I) -> Self {
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
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct InputEventsIter<'a> {
    list: &'a InputEvents<'a>,
    range: Range<u32>,
}

impl<'a> InputEventsIter<'a> {
    #[inline]
    pub(crate) fn new(list: &'a InputEvents<'a>, range: Range<u32>) -> Self {
        Self { list, range }
    }
}

impl<'a> Clone for InputEventsIter<'a> {
    #[inline]
    fn clone(&self) -> Self {
        InputEventsIter {
            list: self.list,
            range: self.range.start..self.range.end,
        }
    }
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

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_any!(InputEvents<'static>: Send, Sync);
}
