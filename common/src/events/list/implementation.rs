use crate::events::TimestampedEvent;

pub trait EventBuffer<'a>: 'a {
    fn size(&self) -> usize;
    fn get(&self, index: usize) -> Option<&TimestampedEvent<'a>>;
    fn push_back(&mut self, event: &TimestampedEvent<'a>);
}

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
