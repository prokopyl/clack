mod core;
mod id;

pub use self::core::*;
pub use id::*;

use crate::events::UnknownEvent;
use std::ffi::CStr;

/// Holds all the possible event types included in a given event space.  
///
/// # Safety
///
/// The implementers of this trait *must* ensure the [`NAME`](EventSpace::NAME) matches the name of
/// the event space coming from the associated CLAP specification.
pub unsafe trait EventSpace<'a>: Sized + 'a {
    /// The specified name of this event space.
    ///
    /// This name *MUST* match the one defined by the CLAP specification for this event space.
    const NAME: &'static CStr;

    /// Casts the given unknown event to the matching event type.
    ///
    /// # Safety
    ///
    /// This method does not take the event space ID into consideration. It is up to the caller
    /// to ensure that the given event does in fact belong to this event space.
    unsafe fn from_unknown(event: &'a UnknownEvent) -> Option<Self>;

    /// Returns this event as a type-erased [`UnknownEvent`].
    fn as_unknown(&self) -> &'a UnknownEvent;
}
