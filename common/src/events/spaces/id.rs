use crate::events::EventSpace;
use crate::events::spaces::core::CoreEventSpace;
use clap_sys::events::CLAP_CORE_EVENT_SPACE_ID;
use std::marker::PhantomData;

/// An event space identifier, optionally tied to its [`EventSpace`] type `S`.
///
/// If `S` is the unit type `()`, then this means this identifier is not tied to any particular
/// event space (i.e. it is type-erased).
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct EventSpaceId<S = ()> {
    id: u16,
    _space: PhantomData<S>,
}

impl<S> EventSpaceId<S> {
    /// The raw numeric value that represents an invalid event space identifier.
    pub const INVALID_ID: u16 = u16::MAX;

    /// Returns raw numeric value of this identifier.
    ///
    /// The returned value cannot be equal to [`Self::INVALID_ID`].
    #[inline]
    pub const fn id(&self) -> u16 {
        self.id
    }

    /// Returns the raw numeric value of the given identifier, or [`Self::INVALID_ID`] if the
    /// `None` is passed.
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
    /// Returns the identifier for the [`CoreEventSpace`].
    ///
    /// It is the only event space that has a constant, specific ID designated by the CLAP
    /// specification, and which can be retrieved without using an event registry.
    #[inline]
    pub const fn core() -> Self {
        Self {
            id: CLAP_CORE_EVENT_SPACE_ID,
            _space: PhantomData,
        }
    }
}

impl EventSpaceId<()> {
    /// Creates a new, type-erased [`EventSpaceId`] from its raw numeric value.
    ///
    /// This returns `None` if the passed value is equal to [`Self::INVALID_ID`].
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
