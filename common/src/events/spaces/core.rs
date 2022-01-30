use crate::events::event_types::NoteOnEvent;
use crate::events::{Event, EventSpace, UnknownEvent};
use std::ffi::CStr;

pub enum CoreEventSpace<'a> {
    NoteOn(&'a NoteOnEvent),
}

unsafe impl<'a> EventSpace<'a> for CoreEventSpace<'a> {
    const NAME: &'static CStr = crate::utils::check_cstr(b"\0");

    unsafe fn from_unknown(event: &'a UnknownEvent<'a>) -> Option<Self> {
        use CoreEventSpace::*;

        match event.header().type_id() {
            NoteOnEvent::TYPE_ID => Some(NoteOn(event.as_event_unchecked())),
            _ => todo!(),
        }
    }

    fn as_unknown(&self) -> &'a UnknownEvent<'a> {
        match self {
            CoreEventSpace::NoteOn(e) => e.as_unknown(),
        }
    }
}
