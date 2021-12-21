use crate::events::list::EventListImplementation;
use crate::events::Event;

pub struct NoopEventList;

impl<'a> EventListImplementation<'a> for NoopEventList {
    #[inline]
    fn size(&self) -> usize {
        0
    }

    #[inline]
    fn get(&self, _index: usize) -> Option<&'a Event> {
        None
    }

    fn push_back(&mut self, _event: &Event) {}
}
