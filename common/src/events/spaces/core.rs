use crate::events::event_types::*;
use crate::events::{Event, EventSpace, UnknownEvent};
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};

/// Core CLAP event set.
///
/// This enum represents the standard events defined in the CLAP specification
/// (e.g. [`NoteOnEvent`], [`ParamValueEvent`]).
///
/// It can be constructed from an [`UnknownEvent`] when the event is one of the core types.
///
/// # Example
/// Converting from [`UnknownEvent`] to a [`CoreEventSpace`]:
/// ```no_run
/// use clack_common::events::UnknownEvent;
/// use clack_common::events::spaces::CoreEventSpace;
///
/// fn handle_event(event: &UnknownEvent) {
///     if let Some(ev) = event.as_core_event() {
///         match ev {
///             CoreEventSpace::NoteOn(note) => { /* handle note‑on */ }
///             CoreEventSpace::ParamValue(param) => { /* handle param change */ }
///             _ => {}
///         }
///     }
/// }
/// ```
#[derive(Copy, Clone, PartialEq)]
pub enum CoreEventSpace<'a> {
    /// A note-on event (key press / attack)
    NoteOn(&'a NoteOnEvent),
    /// A note-off event (key release)
    NoteOff(&'a NoteOffEvent),
    /// A note choke (forcibly ends a note)
    NoteChoke(&'a NoteChokeEvent),
    /// A note end (natural end of note)
    NoteEnd(&'a NoteEndEvent),
    /// A note expression change (e.g. per-note pitch)
    NoteExpression(&'a NoteExpressionEvent),
    /// A parameter value change
    ParamValue(&'a ParamValueEvent),
    /// A parameter modulation change
    ParamMod(&'a ParamModEvent),
    /// Begin of a gesture (automation touch)
    ParamGestureBegin(&'a ParamGestureBeginEvent),
    /// End of a gesture (automation release)
    ParamGestureEnd(&'a ParamGestureEndEvent),
    /// Transport info (playhead, tempo, etc.)
    Transport(&'a TransportEvent),
    /// MIDI 1.0 event
    Midi(&'a MidiEvent),
    /// MIDI 2.0 event
    Midi2(&'a Midi2Event),
    /// MIDI SysEx event
    MidiSysEx(&'a MidiSysExEvent),
}

// SAFETY: The core event space has the empty C string for a name.
unsafe impl<'a> EventSpace<'a> for CoreEventSpace<'a> {
    const NAME: &'static CStr = c"";

    /// Attempts to reinterpret a raw [`UnknownEvent`] as one of the core event types.
    ///
    /// Returns `None` if the event does not belong to this event space.
    unsafe fn from_unknown(event: &'a UnknownEvent) -> Option<Self> {
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
    fn as_unknown(&self) -> &'a UnknownEvent {
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

impl Debug for CoreEventSpace<'_> {
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
