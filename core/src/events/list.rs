use crate::events::Event;
use clap_sys::events::{clap_event, clap_event_list};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};

mod implementation;

pub use implementation::EventListImplementation;
use implementation::NoopEventList;

#[repr(C)]
pub struct EventList<'a> {
    list: clap_event_list,
    _lifetime: PhantomData<&'a mut clap_event_list>,
}

impl<'a> EventList<'a> {
    /// # Safety
    /// The given pointer must be valid for the requested lifetime
    #[inline]
    pub unsafe fn from_raw(raw: *const clap_event_list) -> &'a Self {
        // SAFETY: EventList has the same layout and is repr(C)
        &*(raw as *const _)
    }

    /// # Safety
    /// The given pointer must be valid for the requested lifetime
    #[inline]
    pub unsafe fn from_raw_mut(raw: *const clap_event_list) -> &'a mut Self {
        // SAFETY: EventList has the same layout and is repr(C)
        &mut *(raw as *const _ as *mut _)
    }

    #[inline]
    pub fn as_raw(&self) -> *const clap_event_list {
        &self.list
    }

    #[inline]
    pub fn as_raw_mut(&mut self) -> *mut clap_event_list {
        &mut self.list
    }

    #[inline]
    pub fn no_op() -> Self {
        Self {
            _lifetime: PhantomData,
            list: clap_event_list {
                ctx: ::core::ptr::null_mut(),
                size: size::<NoopEventList>,
                get: get::<NoopEventList>,
                push_back: push_back::<NoopEventList>,
            },
        }
    }

    #[inline]
    pub fn from_implementation<'b: 'a, E: EventListImplementation<'b>>(
        implementation: &'a mut E,
    ) -> Self {
        Self {
            _lifetime: PhantomData,
            list: clap_event_list {
                ctx: implementation as *mut _ as *mut _,
                size: size::<E>,
                get: get::<E>,
                push_back: push_back::<E>,
            },
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { (self.list.size)(&self.list) as usize }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&'a Event<'a>> {
        unsafe {
            (self.list.get)(&self.list, index as u32)
                .as_ref()
                .map(Event::from_raw)
        }
    }

    #[inline]
    pub fn get_mut(&self, index: usize) -> Option<&'a mut Event<'a>> {
        unsafe {
            ((self.list.get)(&self.list, index as u32) as *mut clap_event)
                .as_mut()
                .map(Event::from_raw_mut)
        }
    }

    #[inline]
    pub fn push_back(&mut self, event: &Event) {
        unsafe { (self.list.push_back)(&self.list, event.as_raw()) }
    }

    #[inline]
    pub fn iter(&self) -> EventListIter {
        EventListIter {
            list: self,
            range: 0..self.len(),
        }
    }
}

const INDEX_ERROR: &str = "Indexed EventList out of bounds";

impl<'a> Index<usize> for EventList<'a>
where
    EventList<'a>: 'a,
{
    type Output = Event<'a>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect(INDEX_ERROR)
    }
}

impl<'a> IndexMut<usize> for EventList<'a> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect(INDEX_ERROR)
    }
}

impl<'a> Extend<Event<'a>> for EventList<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = Event<'a>>>(&mut self, iter: T) {
        for event in iter {
            self.push_back(&event)
        }
    }
}

impl<'a: 'e, 'e> Extend<&'e Event<'a>> for EventList<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'e Event<'a>>>(&mut self, iter: T) {
        for event in iter {
            self.push_back(event)
        }
    }
}

impl<'a> IntoIterator for &'a EventList<'a> {
    type Item = &'a Event<'a>;
    type IntoIter = EventListIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, I: EventListImplementation<'a>> From<&'a mut I> for EventList<'a> {
    #[inline]
    fn from(implementation: &'a mut I) -> Self {
        Self::from_implementation(implementation)
    }
}

pub struct EventListIter<'a> {
    list: &'a EventList<'a>,
    range: Range<usize>,
}

impl<'a, 'list> Iterator for EventListIter<'a> {
    type Item = &'a Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().and_then(|i| self.list.get(i))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<'a> ExactSizeIterator for EventListIter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<'a> DoubleEndedIterator for EventListIter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().and_then(|i| self.list.get(i))
    }
}

unsafe extern "C" fn size<'a, E: EventListImplementation<'a>>(list: *const clap_event_list) -> u32 {
    E::size(&*((*list).ctx as *const E)) as u32
}

unsafe extern "C" fn get<'a, E: EventListImplementation<'a>>(
    list: *const clap_event_list,
    index: u32,
) -> *const clap_event {
    E::get_mut(&mut *((*list).ctx as *const _ as *mut E), index as usize)
        .map(|e| e.as_raw() as *const _)
        .unwrap_or_else(::core::ptr::null)
}

unsafe extern "C" fn push_back<'a, E: EventListImplementation<'a>>(
    list: *const clap_event_list,
    event: *const clap_event,
) {
    E::push_back(
        &mut *((*list).ctx as *const _ as *mut E),
        Event::from_raw(&*event),
    )
}
