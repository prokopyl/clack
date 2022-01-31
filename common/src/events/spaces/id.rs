use crate::events::spaces::core::CoreEventSpace;
use crate::events::EventSpace;
use clap_sys::ext::event_registry::CLAP_CORE_EVENT_SPACE_ID;
use std::marker::PhantomData;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct EventSpaceId<S = ()> {
    id: u16,
    _space: PhantomData<S>,
}

impl<S> EventSpaceId<S> {
    #[inline]
    pub const fn id(&self) -> u16 {
        self.id
    }

    #[inline]
    pub const unsafe fn new_unchecked(id: u16) -> Self {
        Self {
            id,
            _space: PhantomData,
        }
    }
}

impl<'a, S: EventSpace<'a>> From<EventSpaceId<S>> for EventSpaceId<()> {
    #[inline]
    fn from(id: EventSpaceId<S>) -> Self {
        Self {
            id: id.id,
            _space: PhantomData,
        }
    }
}

impl<'a> EventSpaceId<CoreEventSpace<'a>> {
    #[inline]
    pub const fn core() -> Self {
        Self {
            id: CLAP_CORE_EVENT_SPACE_ID,
            _space: PhantomData,
        }
    }
}

impl EventSpaceId<()> {
    #[inline]
    pub const fn new(id: u16) -> Option<Self> {
        if id == u16::MAX {
            None
        } else {
            Some(Self {
                id,
                _space: PhantomData,
            })
        }
    }

    #[inline]
    pub const unsafe fn into_unchecked<S>(self) -> EventSpaceId<S> {
        EventSpaceId::new_unchecked(self.id)
    }
}
