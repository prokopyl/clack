use crate::events::event_types::NoteEvent;
use crate::events::{EventSpace, UnknownEvent};
use std::ffi::CStr;

pub enum CoreEventSpace<'a> {
    NoteOn(&'a NoteEvent),
}

unsafe impl<'a> EventSpace for CoreEventSpace<'a> {
    const NAME: &'static CStr = crate::utils::check_cstr(b"\0");

    unsafe fn from_unknown(event: &UnknownEvent) -> Option<Self> {
        todo!()
    }

    fn as_unknown(&self) -> &UnknownEvent {
        todo!()
    }
}
