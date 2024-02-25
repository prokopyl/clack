use crate::events::event_types::*;
use crate::events::{Event, EventSpace, UnknownEvent};
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone, PartialEq)]
pub enum CoreEventSpace<'a> {
    NoteOn(&'a NoteOnEvent),
    NoteOff(&'a NoteOffEvent),
    NoteChoke(&'a NoteChokeEvent),
    NoteEnd(&'a NoteEndEvent),
    NoteExpression(&'a NoteExpressionEvent),
    ParamValue(&'a ParamValueEvent),
    ParamMod(&'a ParamModEvent),
    ParamGestureBegin(&'a ParamGestureBeginEvent),
    ParamGestureEnd(&'a ParamGestureEndEvent),
    Transport(&'a TransportEvent),
    Midi(&'a MidiEvent),
    Midi2(&'a Midi2Event),
    MidiSysEx(&'a MidiSysExEvent<'a>),
}
// SAFETY: there is a null byte in this string.
const EMPTY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"\0") };

// SAFETY: The core event space has the empty C string for a name.
unsafe impl<'a> EventSpace<'a> for CoreEventSpace<'a> {
    const NAME: &'static CStr = EMPTY;

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
            MidiSysExEvent::TYPE_ID => Some(MidiSysEx(event.as_event_unchecked())),
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
            CoreEventSpace::ParamGestureBegin(e) => e.as_unknown(),
            CoreEventSpace::ParamGestureEnd(e) => e.as_unknown(),
        }
    }
}

impl<'a> Debug for CoreEventSpace<'a> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreEventSpace::NoteOn(e) => Debug::fmt(e, f),
            CoreEventSpace::NoteOff(e) => Debug::fmt(e, f),
            CoreEventSpace::NoteChoke(e) => Debug::fmt(e, f),
            CoreEventSpace::NoteEnd(e) => Debug::fmt(e, f),
            CoreEventSpace::NoteExpression(e) => Debug::fmt(e, f),
            CoreEventSpace::ParamValue(e) => Debug::fmt(e, f),
            CoreEventSpace::ParamMod(e) => Debug::fmt(e, f),
            CoreEventSpace::ParamGestureBegin(e) => Debug::fmt(e, f),
            CoreEventSpace::ParamGestureEnd(e) => Debug::fmt(e, f),
            CoreEventSpace::Transport(e) => Debug::fmt(e, f),
            CoreEventSpace::Midi(e) => Debug::fmt(e, f),
            CoreEventSpace::Midi2(e) => Debug::fmt(e, f),
            CoreEventSpace::MidiSysEx(e) => Debug::fmt(e, f),
        }
    }
}
