use crate::events::Event;

pub trait EventListImplementation<'a>: 'a {
    fn size(&self) -> usize;
    fn get_mut(&mut self, index: usize) -> Option<&mut Event<'a>>;
    fn push_back(&mut self, event: &Event<'a>);
}

impl<'a> EventListImplementation<'a> for Vec<Event<'a>> {
    #[inline]
    fn size(&self) -> usize {
        self.len()
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut Event<'a>> {
        <[Event<'a>]>::get_mut(self, index)
    }

    #[inline]
    fn push_back(&mut self, event: &Event<'a>) {
        let closest_event = self.iter().rposition(|e| e.time() <= event.time());
        if let Some(closest_event) = closest_event {
            self.insert(closest_event + 1, *event)
        } else {
            self.push(*event)
        }
    }
}

pub struct NoopEventList;

impl<'a> EventListImplementation<'a> for NoopEventList {
    #[inline]
    fn size(&self) -> usize {
        0
    }

    #[inline]
    fn get_mut(&mut self, _index: usize) -> Option<&mut Event<'a>> {
        None
    }

    fn push_back(&mut self, _event: &Event) {}
}
