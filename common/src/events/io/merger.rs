use crate::events::UnknownEvent;
use std::mem::replace;

/// An iterator that merges two ordered streams of events together.
///
/// This wraps two distinct event iterators and produces all the events produced by both, but in
/// order.
pub struct EventMerger<'a, 'e, I1, I2> {
    iter_1: I1,
    iter_2: I2,

    event_1: Option<&'a UnknownEvent<'e>>,
    event_2: Option<&'a UnknownEvent<'e>>,
    started: bool,
}

impl<'a, 'e, I1, I2> EventMerger<'a, 'e, I1, I2> {
    /// Creates a new event merger from two iterators.
    #[inline]
    pub fn new(iter_1: I1, iter_2: I2) -> Self {
        Self {
            iter_1,
            iter_2,
            event_1: None,
            event_2: None,
            started: false,
        }
    }
}

impl<'a, 'e, I1, I2> Iterator for EventMerger<'a, 'e, I1, I2>
where
    I1: Iterator<Item = &'a UnknownEvent<'e>>,
    I2: Iterator<Item = &'a UnknownEvent<'e>>,
{
    type Item = &'a UnknownEvent<'e>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.event_1 = self.iter_1.next();
            self.event_2 = self.iter_2.next();

            self.started = true;
        }

        match (&self.event_1, &self.event_2) {
            (Some(e1), Some(e2)) if e1.header() <= e2.header() => {
                replace(&mut self.event_1, self.iter_1.next())
            }
            (Some(_), None) => replace(&mut self.event_1, self.iter_1.next()),
            (_, Some(_)) => replace(&mut self.event_2, self.iter_2.next()),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::events::event_types::MidiEvent;
    use crate::events::io::merger::EventMerger;
    use crate::events::{Event, EventFlags, EventHeader};

    #[test]
    fn it_works() {
        let event_0 = MidiEvent::new(EventHeader::new_core(0, EventFlags::empty()), 0, [0; 3]);
        let event_1 = MidiEvent::new(EventHeader::new_core(1, EventFlags::empty()), 0, [1; 3]);
        let event_2 = MidiEvent::new(EventHeader::new_core(2, EventFlags::empty()), 0, [2; 3]);
        let event_3 = MidiEvent::new(EventHeader::new_core(3, EventFlags::empty()), 0, [3; 3]);

        let events_1 = [event_1, event_2];
        let events_2 = [event_0, event_3];

        let mut merger = EventMerger::new(
            events_1.iter().map(|e| e.as_unknown()),
            events_2.iter().map(|e| e.as_unknown()),
        );

        assert_eq!(Some(&event_0), merger.next().unwrap().as_event());
        assert_eq!(Some(&event_1), merger.next().unwrap().as_event());
        assert_eq!(Some(&event_2), merger.next().unwrap().as_event());
        assert_eq!(Some(&event_3), merger.next().unwrap().as_event());
        assert!(merger.next().is_none());
        assert!(merger.next().is_none());
    }
}
