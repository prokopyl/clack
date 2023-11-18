use crate::events::io::{InputEvents, InputEventsIter};
use crate::events::UnknownEvent;
use std::ops::{Bound, Range};

enum State<'a> {
    Started {
        events_len: u32,
    },
    // Other batches: previous_event..next_event
    HasNextEvent {
        events_remaining_range: Range<u32>,
        next_event: &'a UnknownEvent<'a>,
    },
    // Ended
    Ended,
}

pub struct EventBatcher<'a> {
    events: &'a InputEvents<'a>,
    state: State<'a>,
}

impl<'a> EventBatcher<'a> {
    pub(crate) fn new(events: &'a InputEvents<'a>) -> Self {
        Self {
            events,
            state: State::Started {
                events_len: events.len(),
            },
        }
    }
}

impl<'a> Iterator for EventBatcher<'a> {
    type Item = EventBatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::events::io::batcher::State::*;

        match &self.state {
            Ended => None,
            Started { events_len } => {
                let first_event = self.events.get(0);
                let events_len = *events_len;

                match first_event {
                    None => {
                        self.state = Ended;

                        Some(EventBatch {
                            events: InputEventsIter {
                                list: self.events,
                                range: 0..0,
                            },
                            first_sample: 0,
                            next_sample: None,
                        })
                    }
                    Some(event) => {
                        let event_time = event.header().time();
                        if event_time > 0 {
                            self.state = HasNextEvent {
                                next_event: event,
                                events_remaining_range: 1..events_len,
                            };
                            Some(EventBatch {
                                events: InputEventsIter {
                                    list: self.events,
                                    range: 0..0,
                                },
                                first_sample: 0,
                                next_sample: Some(event_time as usize),
                            })
                        } else {
                            let mut lookup_range = 1..events_len;
                            let next_non_matching_event = loop {
                                let Some(next_index) = lookup_range.next() else {
                                    break None;
                                };
                                let Some(next_event) = self.events.get(next_index) else {
                                    continue;
                                };

                                if next_event.header().time() != 0 {
                                    break Some((next_index, next_event));
                                }
                            };

                            match next_non_matching_event {
                                None => {
                                    // Turns out, all events were at index 0. Only one iteration needed.
                                    self.state = Ended;

                                    Some(EventBatch {
                                        events: self.events.iter(),
                                        first_sample: 0,
                                        next_sample: None,
                                    })
                                }
                                Some((event_index, next_event)) => {
                                    self.state = HasNextEvent {
                                        next_event,
                                        events_remaining_range: event_index..events_len,
                                    };

                                    Some(EventBatch {
                                        events: InputEventsIter {
                                            list: self.events,
                                            range: 0..event_index,
                                        },
                                        first_sample: 0,
                                        next_sample: Some(next_event.header().time() as usize),
                                    })
                                }
                            }
                        }
                    }
                }
            }
            HasNextEvent {
                next_event,
                events_remaining_range,
            } => {
                let current_index = events_remaining_range.start;
                let events_len = events_remaining_range.end;
                let mut events_remaining_range = events_remaining_range.clone();

                let current_event_sample_time = next_event.header().time();

                let next_non_matching_event = loop {
                    let Some(next_index) = events_remaining_range.next() else {
                        break None;
                    };
                    let Some(next_event) = self.events.get(next_index) else {
                        continue;
                    };

                    if next_event.header().time() != current_event_sample_time {
                        break Some((next_index, next_event));
                    }
                };

                match next_non_matching_event {
                    None => {
                        self.state = Ended;

                        Some(EventBatch {
                            events: InputEventsIter {
                                list: self.events,
                                range: (current_index - 1)..events_len,
                            },
                            first_sample: current_event_sample_time as usize,
                            next_sample: None,
                        })
                    }
                    Some((event_index, next_event)) => {
                        self.state = HasNextEvent {
                            next_event,
                            events_remaining_range: event_index..events_len,
                        };

                        Some(EventBatch {
                            events: InputEventsIter {
                                list: self.events,
                                range: current_index..event_index,
                            },
                            first_sample: current_event_sample_time as usize,
                            next_sample: Some(next_event.header().time() as usize),
                        })
                    }
                }
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
}
