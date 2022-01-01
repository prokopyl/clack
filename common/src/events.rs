//! Events and related utilities.
//!
//! All events in CLAP are sample-accurate time-stamped events ([`TimestampedEvent`](crate::events::TimestampedEvent)), that are provided to the plugin's
//! audio processor alongside the audio buffers through [`EventList`s](crate::events::EventList).

use crate::events::event_types::*;
use clap_sys::events::{clap_event, clap_event_data, clap_event_type};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

mod list;
pub use list::*;

pub mod event_types;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EventType<'a> {
    NoteOn(NoteEvent),
    NoteOff(NoteEvent),
    NoteEnd(NoteEvent),
    NoteChoke(NoteEvent),
    NoteExpression(NoteExpressionEvent),
    NoteMask(NoteMaskEvent),
    ParamValue(ParamValueEvent),
    ParamMod(ParamModEvent),
    Transport(TransportEvent),
    Midi(MidiEvent),
    MidiSysex(MidiSysexEvent<'a>),
}

impl<'a> EventType<'a> {
    fn from_raw(type_: clap_event_type, data: clap_event_data) -> Option<Self> {
        use clap_sys::events::*;
        use EventType::*;

        unsafe {
            match type_ {
                CLAP_EVENT_NOTE_ON => Some(NoteOn(NoteEvent::from_raw(data.note))),
                CLAP_EVENT_NOTE_OFF => Some(NoteOff(NoteEvent::from_raw(data.note))),
                CLAP_EVENT_NOTE_END => Some(NoteEnd(NoteEvent::from_raw(data.note))),
                CLAP_EVENT_NOTE_CHOKE => Some(NoteChoke(NoteEvent::from_raw(data.note))),
                CLAP_EVENT_NOTE_EXPRESSION => Some(NoteExpression(NoteExpressionEvent::from_raw(
                    data.note_expression,
                ))),
                CLAP_EVENT_NOTE_MASK => Some(NoteMask(NoteMaskEvent::from_raw(data.note_mask))),
                CLAP_EVENT_PARAM_VALUE => {
                    Some(ParamValue(ParamValueEvent::from_raw(data.param_value)))
                }
                CLAP_EVENT_PARAM_MOD => Some(ParamMod(ParamModEvent::from_raw(data.param_mod))),
                CLAP_EVENT_TRANSPORT => Some(Transport(TransportEvent::from_raw(data.time_info))),
                CLAP_EVENT_MIDI => Some(Midi(MidiEvent::from_raw(data.midi))),
                CLAP_EVENT_MIDI_SYSEX => Some(MidiSysex(MidiSysexEvent::from_raw(data.midi_sysex))),

                _ => None,
            }
        }
    }

    fn into_raw(self) -> (clap_event_type, clap_event_data) {
        use clap_sys::events::*;
        use EventType::*;

        match self {
            NoteOn(e) => (CLAP_EVENT_NOTE_ON, clap_event_data { note: e.into_raw() }),
            NoteOff(e) => (CLAP_EVENT_NOTE_OFF, clap_event_data { note: e.into_raw() }),
            NoteEnd(e) => (CLAP_EVENT_NOTE_END, clap_event_data { note: e.into_raw() }),
            NoteChoke(e) => (
                CLAP_EVENT_NOTE_CHOKE,
                clap_event_data { note: e.into_raw() },
            ),
            NoteExpression(e) => (
                CLAP_EVENT_NOTE_EXPRESSION,
                clap_event_data {
                    note_expression: e.into_raw(),
                },
            ),
            NoteMask(e) => (
                CLAP_EVENT_NOTE_MASK,
                clap_event_data {
                    note_mask: e.into_raw(),
                },
            ),
            ParamValue(e) => (
                CLAP_EVENT_PARAM_VALUE,
                clap_event_data {
                    param_value: e.into_raw(),
                },
            ),
            ParamMod(e) => (
                CLAP_EVENT_PARAM_MOD,
                clap_event_data {
                    param_mod: e.into_raw(),
                },
            ),
            Transport(e) => (
                CLAP_EVENT_TRANSPORT,
                clap_event_data {
                    time_info: e.into_raw(),
                },
            ),
            Midi(e) => (CLAP_EVENT_MIDI, clap_event_data { midi: e.into_raw() }),
            MidiSysex(e) => (
                CLAP_EVENT_MIDI_SYSEX,
                clap_event_data {
                    midi_sysex: e.into_raw(),
                },
            ),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TimestampedEvent<'a> {
    inner: clap_event,
    _lifetime: PhantomData<&'a clap_event>, // For MIDI SysEx data
}

impl<'a> TimestampedEvent<'a> {
    #[inline]
    pub fn new(time: u32, event: EventType) -> Self {
        let (type_, data) = event.into_raw();
        Self {
            inner: clap_event { type_, data, time },
            _lifetime: PhantomData,
        }
    }

    #[inline]
    pub fn time(&self) -> u32 {
        self.inner.time
    }

    #[inline]
    pub fn event(&self) -> Option<EventType> {
        EventType::from_raw(self.inner.type_, self.inner.data)
    }

    #[inline]
    pub fn from_raw(event: &clap_event) -> &Self {
        // SAFETY: Event is repr(C) and shares the same memory representation
        unsafe { ::core::mem::transmute(event) }
    }

    #[inline]
    pub fn as_raw(&self) -> &clap_event {
        // SAFETY: Event is repr(C) and shares the same memory representation
        unsafe { ::core::mem::transmute(self) }
    }
}

impl<'a> PartialEq for TimestampedEvent<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.time() == other.time() && self.event() == other.event()
    }
}

impl<'a> PartialOrd for TimestampedEvent<'a> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.time().partial_cmp(&other.time())
    }
}

impl<'a> Debug for TimestampedEvent<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("time", &self.time())
            .field("event", &self.event())
            .finish()
    }
}
