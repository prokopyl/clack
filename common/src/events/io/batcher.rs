use crate::events::io::{InputEvents, InputEventsIter};
use crate::events::UnknownEvent;
use std::ops::Bound;

enum State<'a> {
    Started,
    // Other batches: previous_event..next_event
    HasNextEvent {
        next_event_index: u32,
        next_event: &'a UnknownEvent<'a>,
    },
    // Ended
    Ended,
}

pub struct EventBatcher<'a> {
    events: &'a InputEvents<'a>,
    events_len: u32,
    state: State<'a>,
}

impl<'a> EventBatcher<'a> {
    pub(crate) fn new(events: &'a InputEvents<'a>) -> Self {
        Self {
            events,
            events_len: events.len(),
            state: State::Started,
        }
    }
}

impl<'a> Iterator for EventBatcher<'a> {
    type Item = EventBatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::events::io::batcher::State::*;

        let (next_event_index, next_event_sample_time) = match self.state {
            Ended => return None,
            Started => match self.events.get(0) {
                None => {
                    self.state = Ended;

                    return Some(EventBatch {
                        events: InputEventsIter::new(self.events, 0..0),
                        first_sample: 0,
                        next_sample: None,
                    });
                }
                Some(first_event) => {
                    let event_time = first_event.header().time();
                    if event_time == 0 {
                        (0, 0)
                    } else {
                        self.state = HasNextEvent {
                            next_event: first_event,
                            next_event_index: 0,
                        };
                        return Some(EventBatch {
                            events: InputEventsIter::new(self.events, 0..0),
                            first_sample: 0,
                            next_sample: Some(event_time as usize),
                        });
                    }
                }
            },
            HasNextEvent {
                next_event,
                next_event_index,
            } => (next_event_index, next_event.header().time()),
        };

        let mut next_non_matching_event = None;

        for next_index in (next_event_index + 1)..self.events.len() {
            let Some(next_event) = self.events.get(next_index) else {
                continue;
            };

            if next_event.header().time() != next_event_sample_time {
                next_non_matching_event = Some((next_index, next_event));
                break;
            }
        }

        match next_non_matching_event {
            None => {
                self.state = Ended;

                Some(EventBatch {
                    events: InputEventsIter::new(self.events, next_event_index..self.events_len),
                    first_sample: next_event_sample_time as usize,
                    next_sample: None,
                })
            }
            Some((event_index, next_event)) => {
                self.state = HasNextEvent {
                    next_event,
                    next_event_index: event_index,
                };

                Some(EventBatch {
                    events: InputEventsIter::new(self.events, next_event_index..event_index),
                    first_sample: next_event_sample_time as usize,
                    next_sample: Some(next_event.header().time() as usize),
                })
            }
        }
    }
}

pub struct EventBatch<'a> {
    events: InputEventsIter<'a>,
    first_sample: usize,
    next_sample: Option<usize>,
}

impl<'a> EventBatch<'a> {
    #[inline]
    pub fn events_iter(&self) -> InputEventsIter<'a> {
        self.events.clone()
    }

    #[inline]
    pub fn first_sample(&self) -> usize {
        self.first_sample
    }

    #[inline]
    pub fn next_sample(&self) -> Option<usize> {
        self.next_sample
    }

    #[inline]
    pub fn sample_bounds(&self) -> (Bound<usize>, Bound<usize>) {
        (
            Bound::Included(self.first_sample),
            match self.next_sample {
                None => Bound::Unbounded,
                Some(end) => Bound::Excluded(end),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event_types::ParamGestureBeginEvent;
    use crate::events::{EventFlags, EventHeader};

    #[test]
    pub fn works_with_empty_events() {
        let buf: [&UnknownEvent; 0] = [];
        let events = InputEvents::from_buffer(&buf);
        let mut events = events.batch();

        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 0);
            assert_eq!(batch.next_sample(), None);

            let mut batch_events = batch.events_iter();
            assert!(batch_events.next().is_none());
        }

        assert!(events.next().is_none())
    }

    #[test]
    pub fn works_with_single_zero_event() {
        let buf = [ParamGestureBeginEvent::new(
            EventHeader::new_core(0, EventFlags::empty()),
            0,
        )];

        let events = InputEvents::from_buffer(&buf);
        let mut events = events.batch();

        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 0);
            assert_eq!(batch.next_sample(), None);

            let mut batch_events = batch.events_iter();
            assert_eq!(&buf[0], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }

        assert!(events.next().is_none())
    }

    #[test]
    pub fn works_with_single_nonzero_event() {
        let buf = [ParamGestureBeginEvent::new(
            EventHeader::new_core(5, EventFlags::empty()),
            0,
        )];

        let events = InputEvents::from_buffer(&buf);
        let mut events = events.batch();

        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 0);
            assert_eq!(batch.next_sample(), Some(5));

            let mut batch_events = batch.events_iter();
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 5);
            assert_eq!(batch.next_sample(), None);

            let mut batch_events = batch.events_iter();
            assert_eq!(&buf[0], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }

        assert!(events.next().is_none())
    }

    #[test]
    pub fn works_with_two_grouped_nonzero_events() {
        let buf = [
            ParamGestureBeginEvent::new(EventHeader::new_core(5, EventFlags::empty()), 0),
            ParamGestureBeginEvent::new(EventHeader::new_core(5, EventFlags::empty()), 0),
        ];

        let events = InputEvents::from_buffer(&buf);
        let mut events = events.batch();

        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 0);
            assert_eq!(batch.next_sample(), Some(5));

            let mut batch_events = batch.events_iter();
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 5);
            assert_eq!(batch.next_sample(), None);

            let mut batch_events = batch.events_iter();
            assert_eq!(&buf[0], batch_events.next().unwrap().as_event().unwrap());
            assert_eq!(&buf[1], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }

        assert!(events.next().is_none())
    }

    #[test]
    pub fn works_with_two_distinct_nonzero_events() {
        let buf = [
            ParamGestureBeginEvent::new(EventHeader::new_core(5, EventFlags::empty()), 0),
            ParamGestureBeginEvent::new(EventHeader::new_core(10, EventFlags::empty()), 0),
        ];

        let events = InputEvents::from_buffer(&buf);
        let mut events = events.batch();

        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 0);
            assert_eq!(batch.next_sample(), Some(5));

            let mut batch_events = batch.events_iter();
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 5);
            assert_eq!(batch.next_sample(), Some(10));

            let mut batch_events = batch.events_iter();
            assert_eq!(&buf[0], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 10);
            assert_eq!(batch.next_sample(), None);

            let mut batch_events = batch.events_iter();
            assert_eq!(&buf[1], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }

        assert!(events.next().is_none())
    }

    #[test]
    pub fn three_distinct_nonzero_events() {
        let buf = [
            ParamGestureBeginEvent::new(EventHeader::new_core(5, EventFlags::empty()), 0),
            ParamGestureBeginEvent::new(EventHeader::new_core(10, EventFlags::empty()), 0),
            ParamGestureBeginEvent::new(EventHeader::new_core(15, EventFlags::empty()), 0),
        ];

        let events = InputEvents::from_buffer(&buf);
        let mut events = events.batch();

        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 0);
            assert_eq!(batch.next_sample(), Some(5));

            let mut batch_events = batch.events_iter();
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 5);
            assert_eq!(batch.next_sample(), Some(10));

            let mut batch_events = batch.events_iter();
            assert_eq!(&buf[0], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 10);
            assert_eq!(batch.next_sample(), Some(15));

            let mut batch_events = batch.events_iter();
            assert_eq!(&buf[1], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 15);
            assert_eq!(batch.next_sample(), None);

            let mut batch_events = batch.events_iter();
            assert_eq!(&buf[2], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }

        assert!(events.next().is_none())
    }
}
