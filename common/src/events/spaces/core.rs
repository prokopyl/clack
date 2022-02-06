use crate::events::event_types::*;
use crate::events::{Event, EventSpace, UnknownEvent};
use std::ffi::CStr;

pub enum CoreEventSpace<'a> {
    NoteOn(&'a NoteOnEvent),
    NoteOff(&'a NoteOffEvent),
    NoteChoke(&'a NoteChokeEvent),
    NoteEnd(&'a NoteEndEvent),
    NoteExpression(&'a NoteExpressionEvent),
    ParamValue(&'a ParamValueEvent),
    ParamMod(&'a ParamModEvent),
    Transport(&'a TransportEvent),
    Midi(&'a MidiEvent),
    Midi2(&'a Midi2Event),
    MidiSysEx(&'a MidiSysexEvent<'a>),
}

unsafe impl<'a> EventSpace<'a> for CoreEventSpace<'a> {
    const NAME: &'static CStr = crate::utils::check_cstr(b"\0");

    unsafe fn from_unknown(event: &'a UnknownEvent<'a>) -> Option<Self> {
        use CoreEventSpace::*;

        match event.header().type_id() {
            NoteOnEvent::TYPE_ID => Some(NoteOn(event.as_event_unchecked())),
            NoteOffEvent::TYPE_ID => Some(NoteOff(event.as_event_unchecked())),
            NoteChokeEvent::TYPE_ID => Some(NoteChoke(event.as_event_unchecked())),
            NoteEndEvent::TYPE_ID => Some(NoteEnd(event.as_event_unchecked())),
            NoteExpressionEvent::TYPE_ID => Some(NoteExpression(event.as_event_unchecked())),
            ParamValueEvent::TYPE_ID => Some(ParamValue(event.as_event_unchecked())),
            ParamModEvent::TYPE_ID => Some(ParamMod(event.as_event_unchecked())),
            TransportEvent::TYPE_ID => Some(Transport(event.as_event_unchecked())),
            MidiEvent::TYPE_ID => Some(Midi(event.as_event_unchecked())),
            Midi2Event::TYPE_ID => Some(Midi2(event.as_event_unchecked())),
            MidiSysexEvent::TYPE_ID => Some(MidiSysEx(event.as_event_unchecked())),
            _ => None,
        }
    }

    #[inline]
    fn as_unknown(&self) -> &'a UnknownEvent<'a> {
        match self {
            CoreEventSpace::NoteOn(e) => e.as_unknown(),
            CoreEventSpace::NoteOff(e) => e.as_unknown(),
            CoreEventSpace::NoteChoke(e) => e.as_unknown(),
            CoreEventSpace::NoteEnd(e) => e.as_unknown(),
            CoreEventSpace::NoteExpression(e) => e.as_unknown(),
            CoreEventSpace::ParamValue(e) => e.as_unknown(),
            CoreEventSpace::ParamMod(e) => e.as_unknown(),
            CoreEventSpace::Transport(e) => e.as_unknown(),
            CoreEventSpace::Midi(e) => e.as_unknown(),
            CoreEventSpace::Midi2(e) => e.as_unknown(),
            CoreEventSpace::MidiSysEx(e) => e.as_unknown(),
        }
    }
}
