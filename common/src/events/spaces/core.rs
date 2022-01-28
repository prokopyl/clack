use crate::events::event_types::NoteEvent;
use crate::events::EventSpace;
use std::ffi::CStr;

pub enum CoreEventSpace<'a> {
    NoteOn(&'a NoteEvent),
}

unsafe impl<'a> EventSpace for CoreEventSpace<'a> {
    const NAME: &'static CStr = crate::utils::check_cstr(b"\0");
}
