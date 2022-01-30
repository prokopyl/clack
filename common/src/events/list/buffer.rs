use crate::events::list::implementation::{InputEventBuffer, OutputEventBuffer};
use crate::events::{Event, UnknownEvent};
use clap_sys::events::clap_event_header;
use core::mem::{size_of_val, MaybeUninit};
use core::slice::from_raw_parts_mut;
use std::ops::Range;

pub struct EventBuffer {
    headers: Vec<MaybeUninit<clap_event_header>>,
    indexes: Vec<u32>,
}

#[inline]
pub(crate) fn byte_index_to_value_index<T>(size: usize) -> usize {
    let type_size = ::core::mem::size_of::<T>();
    if type_size == 0 {
        0
    } else {
        size / type_size + if size % type_size > 0 { 1 } else { 0 }
    }
}

impl EventBuffer {
    pub fn with_capacity(event_headers: usize) -> Self {
        Self {
            headers: Vec::with_capacity(event_headers),
            indexes: Vec::with_capacity(event_headers),
        }
    }

    pub fn clear(&mut self) {
        self.indexes.clear();
        self.headers.clear();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indexes.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> EventBufferIter {
        EventBufferIter {
            buffer: self,
            range: 0..self.len(),
        }
    }

    pub fn fill_with<'a, E: Event<'a>>(&mut self, events: impl IntoIterator<Item = &'a E>) {
        for e in events {
            self.push_back(e.as_unknown())
        }
    }

    fn allocate_mut(&mut self, byte_size: usize) -> &mut [u8] {
        let previous_len = self.headers.len();
        let headers_size = byte_index_to_value_index::<clap_event_header>(byte_size);
        self.headers
            .resize(previous_len + headers_size, MaybeUninit::zeroed());

        // PANIC: we just resized, this should not panic unless there is a bug in the implementation
        let new_elements = &mut self.headers[previous_len..];

        // SAFETY: casting anything to bytes is always safe
        let new_bytes = unsafe {
            from_raw_parts_mut(
                new_elements.as_mut_ptr() as *mut u8,
                size_of_val(new_elements),
            )
        };

        // PANIC: this should not panic unless there is a bug in the implementation
        &mut new_bytes[..byte_size]
    }
}

impl InputEventBuffer for EventBuffer {
    #[inline]
    fn len(&self) -> u32 {
        self.indexes.len() as u32
    }

    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        let header_index = (*self.indexes.get(index as usize)?) as usize;
        // SAFETY: Registered indexes always have actual event headers written by push_back
        // PANIC: We used registered indexes, this should never panic
        let event = unsafe { self.headers[header_index].assume_init_ref() };

        // SAFETY: the event header was written from a valid UnknownEvent in push_back
        Some(unsafe { UnknownEvent::from_raw(event) })
    }
}

impl OutputEventBuffer for EventBuffer {
    fn push_back(&mut self, event: &UnknownEvent) {
        let index = self.headers.len();
        let event_bytes = event.as_bytes();
        let bytes = self.allocate_mut(event_bytes.len());

        // PANIC: bytes is guaranteed by allocate_mut to be just the right size
        bytes.copy_from_slice(event_bytes);
        self.indexes.push(index as u32);
    }
}

pub struct EventBufferIter<'a> {
    buffer: &'a EventBuffer,
    range: Range<usize>,
}

impl<'a> Iterator for EventBufferIter<'a> {
    type Item = &'a UnknownEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let next_index = self.range.next()?;
        self.buffer.get(next_index as u32)
    }
}
