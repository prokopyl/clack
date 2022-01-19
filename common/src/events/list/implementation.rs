use crate::events::TimestampedEvent;

/// Represents an ordered buffer of [`TimestampedEvent`s](crate::events::TimestampedEvent).
///
/// Any type implementing this trait can be turned into an [`EventList`](crate::events::EventList),
/// through the [`from_buffer`](crate::events::EventList::from_buffer) function.
///
/// # Example
///
/// The following example is the implementation of `EventBuffer` for a `Vec`:
///
/// ```
/// # struct Vec<T>(T); // Can only implement trait for local types :c
/// # impl<T> Vec<T> { pub fn len() -> usize { todo!() } pub fn insert(&mut self, pos: usize, t: T) { todo!() } pub fn push(&mut self, t: T) { todo!() } }
/// # impl<T> core::ops::Deref for Vec<T> {type Target = [T]; fn deref(&self) -> &Self::Target { todo!() }}
/// use clack_common::events::{EventBuffer, TimestampedEvent};
///
/// impl<'a> EventBuffer<'a> for Vec<TimestampedEvent<'a>> {
///     fn size(&self) -> usize {
///         self.len()
///     }
///
///     fn get(&self, index: usize) -> Option<&TimestampedEvent<'a>> {
///         <[TimestampedEvent<'a>]>::get(self, index)
///     }
///
///     // Keeps events in order
///     fn push_back(&mut self, event: &TimestampedEvent<'a>) {
///         let closest_event = self.iter().rposition(|e| e.time() <= event.time());
///         if let Some(closest_event) = closest_event {
///             self.insert(closest_event + 1, *event)
///         } else {
///             self.push(*event)
///         }
///     }
/// }
/// ```
pub trait EventBuffer<'a>: 'a {
    /// Returns the number of events in the buffer.
    fn size(&self) -> usize;
    /// Returns an immutable reference to the event present at the given index, or `None` if the
    /// index is out of bounds.
    fn get(&self, index: usize) -> Option<&TimestampedEvent<'a>>;
    /// Appends a copy of the given event to the buffer.
    ///
    /// The event may not be always pushed at the end of the list, as implementations may choose to
    /// reorder events internally so that events are properly ordered by timestamp.
    fn push_back(&mut self, event: &TimestampedEvent<'a>);
}

/// An implementation of an event buffer for a `Vec`.
///
/// This implementation ensures that new events are always inserted in order relative to other events.
impl<'a> EventBuffer<'a> for Vec<TimestampedEvent<'a>> {
    #[inline]
    fn size(&self) -> usize {
        self.len()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&TimestampedEvent<'a>> {
        <[TimestampedEvent<'a>]>::get(self, index)
    }

    #[inline]
    fn push_back(&mut self, event: &TimestampedEvent<'a>) {
        let closest_event = self.iter().rposition(|e| e.time() <= event.time());
        if let Some(closest_event) = closest_event {
            self.insert(closest_event + 1, *event)
        } else {
            self.push(*event)
        }
    }
}

impl<'a> EventBuffer<'a> for [TimestampedEvent<'a>] {
    #[inline]
    fn size(&self) -> usize {
        self.len()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<&TimestampedEvent<'a>> {
        <[TimestampedEvent<'a>]>::get(self, index)
    }

    #[inline]
    fn push_back(&mut self, _event: &TimestampedEvent<'a>) {
        eprintln!("[WARN] Attempted to push an event on a read-only event buffer!")
    }
}
