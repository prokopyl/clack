use crate::events::io::{InputEvents, InputEventsIter};
use std::ops::Bound;

#[derive(Copy, Clone, Debug)]
enum State {
    Started {
        first_event_sample_time: Option<u32>,
    },
    HasNextEvent {
        next_event_index: u32,
        next_event_sample_time: u32,
    },
    Ended,
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct EventBatcher<'a> {
    events: &'a InputEvents<'a>,
    events_len: u32,
    state: State,
}

impl<'a> EventBatcher<'a> {
    pub(crate) fn new(events: &'a InputEvents<'a>) -> Self {
        Self {
            events,
            events_len: events.len(),
            state: State::Started {
                first_event_sample_time: events.get(0).map(|e| e.header().time()),
            },
        }
    }

    fn next_non_matching(
        &self,
        current_event_index: u32,
        current_sample: u32,
    ) -> Option<(u32, u32)> {
        for next_index in (current_event_index + 1)..self.events_len {
            let Some(next_event) = self.events.get(next_index) else {
                continue;
            };

            let next_event_sample_time = next_event.header().time();
            if next_event_sample_time != current_sample {
                return Some((next_index, next_event_sample_time));
            }
        }

        None
    }
}

impl<'a> Iterator for EventBatcher<'a> {
    type Item = EventBatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::events::io::batcher::State::*;

        let (current_event_index, current_sample, next_non_matching_event) = match self.state {
            Ended => return None,
            HasNextEvent {
                next_event_index,
                next_event_sample_time,
            } => (
                next_event_index,
                next_event_sample_time,
                self.next_non_matching(next_event_index, next_event_sample_time),
            ),
            Started {
                first_event_sample_time: Some(0),
            } => (0, 0, self.next_non_matching(0, 0)),
            Started {
                first_event_sample_time: None,
            } => (0, 0, None),
            Started {
                first_event_sample_time: Some(first_event_sample_time),
            } => (0, 0, Some((0, first_event_sample_time))),
        };

        match next_non_matching_event {
            None => {
                self.state = Ended;

                Some(EventBatch {
                    events: InputEventsIter::new(self.events, current_event_index..self.events_len),
                    first_sample: current_sample as usize,
                    next_batch_first_sample: None,
                })
            }
            Some((next_event_index, next_event_sample_time)) => {
                self.state = HasNextEvent {
                    next_event_sample_time,
                    next_event_index,
                };

                Some(EventBatch {
                    events: InputEventsIter::new(
                        self.events,
                        current_event_index..next_event_index,
                    ),
                    first_sample: current_sample as usize,
                    next_batch_first_sample: Some(next_event_sample_time as usize),
                })
            }
        }
    }
}

pub struct EventBatch<'a> {
    events: InputEventsIter<'a>,
    first_sample: usize,
    next_batch_first_sample: Option<usize>,
}

impl<'a> EventBatch<'a> {
    #[inline]
    pub fn events(&self) -> InputEventsIter<'a> {
        self.events.clone()
    }

    #[inline]
    pub fn first_sample(&self) -> usize {
        self.first_sample
    }

    #[inline]
    pub fn next_batch_first_sample(&self) -> Option<usize> {
        self.next_batch_first_sample
    }

    #[inline]
    pub fn sample_bounds(&self) -> (Bound<usize>, Bound<usize>) {
        (
            Bound::Included(self.first_sample),
            match self.next_batch_first_sample {
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
    use crate::events::{EventFlags, EventHeader, UnknownEvent};

    #[test]
    pub fn works_with_empty_events() {
        let buf: [&UnknownEvent; 0] = [];
        let events = InputEvents::from_buffer(&buf);
        let mut events = events.batch();

        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 0);
            assert_eq!(batch.next_batch_first_sample(), None);

            let mut batch_events = batch.events();
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
            assert_eq!(batch.next_batch_first_sample(), None);

            let mut batch_events = batch.events();
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
            assert_eq!(batch.next_batch_first_sample(), Some(5));

            let mut batch_events = batch.events();
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 5);
            assert_eq!(batch.next_batch_first_sample(), None);

            let mut batch_events = batch.events();
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
            assert_eq!(batch.next_batch_first_sample(), Some(5));

            let mut batch_events = batch.events();
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 5);
            assert_eq!(batch.next_batch_first_sample(), None);

            let mut batch_events = batch.events();
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
            assert_eq!(batch.next_batch_first_sample(), Some(5));

            let mut batch_events = batch.events();
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 5);
            assert_eq!(batch.next_batch_first_sample(), Some(10));

            let mut batch_events = batch.events();
            assert_eq!(&buf[0], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 10);
            assert_eq!(batch.next_batch_first_sample(), None);

            let mut batch_events = batch.events();
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
            assert_eq!(batch.next_batch_first_sample(), Some(5));

            let mut batch_events = batch.events();
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 5);
            assert_eq!(batch.next_batch_first_sample(), Some(10));

            let mut batch_events = batch.events();
            assert_eq!(&buf[0], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 10);
            assert_eq!(batch.next_batch_first_sample(), Some(15));

            let mut batch_events = batch.events();
            assert_eq!(&buf[1], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }
        {
            let batch = events.next().unwrap();
            assert_eq!(batch.first_sample(), 15);
            assert_eq!(batch.next_batch_first_sample(), None);

            let mut batch_events = batch.events();
            assert_eq!(&buf[2], batch_events.next().unwrap().as_event().unwrap());
            assert!(batch_events.next().is_none());
        }

        assert!(events.next().is_none())
    }
}
