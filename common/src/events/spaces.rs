pub mod core;
mod id;

pub use id::*;

use crate::events::UnknownEvent;
use std::ffi::CStr;

pub unsafe trait EventSpace: Sized {
    const NAME: &'static CStr;

    unsafe fn from_unknown(event: &UnknownEvent) -> Option<Self>;
    fn as_unknown(&self) -> &UnknownEvent;
}
