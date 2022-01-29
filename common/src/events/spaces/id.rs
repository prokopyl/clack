use crate::events::core::CoreEventSpace;
use crate::events::EventSpace;
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

impl<S: EventSpace> From<EventSpaceId<S>> for EventSpaceId<()> {
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
            id: 0,
            _space: PhantomData,
        } // TODO: CLAP_CORE_EVENT_SPACE_ID clap_sys
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
}
