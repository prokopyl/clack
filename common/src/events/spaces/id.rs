use crate::events::EventSpace;
use crate::events::spaces::core::CoreEventSpace;
use clap_sys::events::CLAP_CORE_EVENT_SPACE_ID;
use std::marker::PhantomData;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct EventSpaceId<S = ()> {
    id: u16,
    _space: PhantomData<S>,
}

impl<S> EventSpaceId<S> {
    pub const INVALID_ID: u16 = u16::MAX;

    #[inline]
    pub const fn id(&self) -> u16 {
        self.id
    }

    #[inline]
    pub const fn optional_id(this: &Option<Self>) -> u16 {
        match this {
            Some(i) => i.id,
            None => u16::MAX,
        }
    }

    /// Gets an event space ID from a numerical value, without performing any checks.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure the event type is properly associated to the event space `S`.
    /// The caller must also ensure the ID is not equal to [`Self::INVALID_ID`].
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

impl EventSpaceId<CoreEventSpace<'_>> {
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

    /// Turns this unassociated event space ID into one associated with the given event space.
    ///
    /// # Safety
    ///
    /// The caller *must* ensure the event type is properly associated to the event space `S`.
    #[inline]
    pub const unsafe fn into_unchecked<S>(self) -> EventSpaceId<S> {
        EventSpaceId::new_unchecked(self.id)
    }
}
