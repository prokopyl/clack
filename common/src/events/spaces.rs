mod core;
mod id;

pub use self::core::*;
pub use id::*;

use crate::events::UnknownEvent;
use std::ffi::CStr;

pub unsafe trait EventSpace<'a>: Sized + 'a {
    const NAME: &'static CStr;

    unsafe fn from_unknown(event: &'a UnknownEvent<'a>) -> Option<Self>;
    fn as_unknown(&self) -> &'a UnknownEvent<'a>;
}
