use crate::events::io::implementation::{InputEventBuffer, OutputEventBuffer};
use crate::events::io::TryPushError;
use crate::events::spaces::CoreEventSpace;
use crate::events::UnknownEvent;
use clap_sys::events::clap_event_header;
use core::mem::{size_of_val, MaybeUninit};
use core::slice::from_raw_parts_mut;
use std::ops::Range;

#[repr(C, align(8))]
#[derive(Copy, Clone)]
struct AlignedEventHeader(clap_event_header);

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
    #[inline]
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            indexes: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(event_headers: usize) -> Self {
        Self {
            headers: Vec::with_capacity(
                event_headers * core::mem::size_of::<CoreEventSpace<'static>>(),
            ),
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
    pub fn get(&self, index: u32) -> Option<&UnknownEvent> {
        <Self as InputEventBuffer>::get(self, index)
    }

    #[inline]
    pub fn iter(&self) -> EventBufferIter {
        EventBufferIter {
            buffer: self,
            range: 0..self.len(),
        }
    }

    pub fn sort(&mut self) {
        self.indexes.sort_by_key(|i| {
            // SAFETY: Registered indexes always have actual event headers written by append_header_data
            // PANIC: We used registered indexes, this should never panic
            let event = unsafe { self.headers[*i as usize].assume_init_ref() };
            event.0.time
        })
    }

    pub fn insert(&mut self, event: &UnknownEvent<'static>, position: usize) {
        let index = self.append_header_data(event);
        self.indexes.insert(position, index as u32);
    }

    pub fn push_all<'a>(&mut self, events: impl IntoIterator<Item = &'a UnknownEvent<'static>>) {
        for e in events {
            self.push(e);
        }
    }

    pub fn push(&mut self, event: &UnknownEvent<'static>) {
        let index = self.append_header_data(event);
        self.indexes.push(index as u32);
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
            from_raw_parts_mut(
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

impl InputEventBuffer for EventBuffer {
    #[inline]
    fn len(&self) -> u32 {
        self.indexes.len() as u32
    }

    fn get(&self, index: u32) -> Option<&UnknownEvent> {
        let header_index = (*self.indexes.get(index as usize)?) as usize;
        // SAFETY: Registered indexes always have actual event headers written by append_header_data
        // PANIC: We used registered indexes, this should never panic
        let event = unsafe { self.headers[header_index].assume_init_ref() };
        let event_size = event.0.size as usize;
        let header_end_index =
            byte_index_to_value_index::<AlignedEventHeader>(event_size) + header_index;

        let event = &self.headers[header_index..header_end_index];

        let event_bytes_padded = unsafe {
            core::slice::from_raw_parts(
                event.as_ptr() as *const _,
                event.len() * core::mem::size_of::<AlignedEventHeader>(),
            )
        };

        let event_bytes = &event_bytes_padded[..event_size];

        // SAFETY: the event header was written from a valid UnknownEvent in append_header_data
        Some(unsafe { UnknownEvent::from_bytes_unchecked(event_bytes) })
    }
}

impl OutputEventBuffer for EventBuffer {
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
