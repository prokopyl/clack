use crate::events::io::implementation::{raw_output_events, OutputEventBuffer};
use crate::events::io::void_output_events;
use crate::events::UnknownEvent;
use clap_sys::events::clap_output_events;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// An ordered list of timestamped events.
///
/// `OutputEvents`s are always ordered: an event at index `i` will always have a timestamp smaller than
/// or equal to the timestamp of the next event at index `i + 1`.
///
/// `OutputEvents`s do not own the event data, they are only lightweight wrappers around a compatible
/// event buffer (i.e. [`OutputEventBuffer`]), see [`OutputEvents::from_buffer`].
///
/// Unlike [`Vec`s](Vec) or slices, `OutputEvents`s only support appending a new event to
/// the list ([`try_push`](OutputEvents::try_push)).
///
/// This type also implements a few extra features based on these operations for convenience,
/// such as the [`Extend`](Extend) trait.
///
/// # Example
///```
/// use clack_common::events::{Event, EventHeader};
/// use clack_common::events::event_types::{NoteEvent, NoteOnEvent};
/// use clack_common::events::io::{EventBuffer, OutputEvents};
///
/// let mut buf = EventBuffer::new();
/// let mut output_events = OutputEvents::from_buffer(&mut buf);
///
/// let event = NoteOnEvent(NoteEvent::new(EventHeader::new(0), 60, 0, 12, 0, 4.2));
/// output_events.try_push(event.as_unknown()).unwrap();
///
/// assert_eq!(1, buf.len());
/// ```
#[repr(C)]
pub struct OutputEvents<'a> {
    inner: clap_output_events,
    _lifetime: PhantomData<(&'a clap_output_events, *const ())>,
}

impl<'a> OutputEvents<'a> {
    /// Creates a mutable reference to an OutputEvents list from a given C FFI-compatible pointer.
    ///
    /// # Safety
    /// The caller must ensure the given pointer is valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_raw_mut(raw: &mut clap_output_events) -> &'a mut Self {
        // SAFETY: OutputEvents list has the same layout and is repr(C)
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
    /// This buffer may have been used by a previous `OutputEvents`, and will keep all events that
    /// have been [`push`ed](OutputEvents::try_push) into it. This allows to reuse event buffers
    /// between process trips.
    ///
    /// # Realtime Safety
    ///
    /// This is a very cheap, realtime-safe operation that does not change the given buffer in any
    /// way. However, since users of the `OutputEvents` may call `append` on it, the buffer should have
    /// a reasonable amount of storage pre-allocated, as to not have to perform additional
    /// allocations on the audio thread.
    ///
    /// # Example
    /// ```
    /// use clack_common::events::io::{EventBuffer, OutputEvents};
    ///
    /// // Allocate the buffer on the main thread
    /// let mut buf = EventBuffer::new();
    ///
    /// // Later, on the DSP thread
    /// let output_events = OutputEvents::from_buffer(&mut buf);
    /// ```
    #[inline]
    pub fn from_buffer<'b: 'a, O: OutputEventBuffer<'b>>(buffer: &'a mut O) -> Self {
        Self {
            inner: raw_output_events(buffer),
            _lifetime: PhantomData,
        }
    }

    /// Creates a void "list" which ignores every event that is pushed to it.
    ///
    /// This can be useful if you do not intend to support output events at all.
    #[inline]
    pub const fn void() -> Self {
        Self {
            inner: void_output_events(),
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
    /// # Errors
    ///
    /// This method will return a [`TryPushError`] if the event could not be pushed to the list.
    ///
    /// The exact reason is left at the implementer's discretion, but this is usually a sign that
    /// the implementer ran out of buffer space, and either cannot or refuses to allocate more.
    ///
    /// # Realtime Safety
    ///
    /// This operation may cause the underlying event buffer to be reallocated by the host, therefore
    /// realtime safety cannot be guaranteed. However, it is expected for hosts to pre-allocate a
    /// reasonable amount of space before forwarding the list to the plugin, in order to make
    /// allocations as unlikely as possible.
    #[inline]
    pub fn try_push<E: AsRef<UnknownEvent<'a>>>(&mut self, event: E) -> Result<(), TryPushError> {
        let try_push = self.inner.try_push.ok_or(TryPushError)?;

        if !unsafe { try_push(&self.inner, event.as_ref().as_raw()) } {
            Err(TryPushError {})
        } else {
            Ok(())
        }
    }
}

/// An error that may occur when [`OutputEvents::try_push`] couldn't complete.
///
/// See the documentation of [`OutputEvents::try_push`] for more information.
#[non_exhaustive]
#[derive(Debug, Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct TryPushError;

impl TryPushError {
    /// Creates a new [`TryPushError`].
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Display for TryPushError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to push event into output event buffer")
    }
}

impl Error for TryPushError {}

impl<'a, I: OutputEventBuffer<'a>> From<&'a mut I> for OutputEvents<'a> {
    #[inline]
    fn from(implementation: &'a mut I) -> Self {
        Self::from_buffer(implementation)
    }
}

impl<'a: 'b, 'b> Extend<&'b UnknownEvent<'a>> for OutputEvents<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'b UnknownEvent<'a>>>(&mut self, iter: T) {
        #[allow(unused_must_use)]
        for event in iter {
            self.try_push(event);
        }
    }
}

#[derive(Copy, Clone)]
struct VoidEvents;
impl<'a> OutputEventBuffer<'a> for VoidEvents {
    #[inline]
    fn try_push(&mut self, _event: &UnknownEvent<'a>) -> Result<(), TryPushError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_any!(OutputEvents<'static>: Send, Sync);
}
