use crate::events::event_types::note::NoteEvent;
use crate::events::event_types::note_expression::NoteExpressionEvent;
use clap_sys::events::clap_event;

pub mod list;

pub mod event_match;
pub mod event_types;

pub enum EventType {
    NoteOn(NoteEvent),
    NoteOff(NoteEvent),
    NoteEnd(NoteEvent),
    NoteChoke(NoteEvent),
    NoteExpression(NoteExpressionEvent),

    Unknown,
}

pub struct Event {
    pub timestamp: u32,
    pub event_type: EventType,
}

impl Event {
    pub fn from_raw_ref(_event: &clap_event) -> &Self {
        todo!()
    }

    pub fn from_raw(event: &clap_event) -> Self {
        use clap_sys::events::*;

        let event_type = unsafe {
            match event.type_ {
                CLAP_EVENT_NOTE_ON => EventType::NoteOn(NoteEvent::from_raw(event.data.note)),
                _ => EventType::Unknown,
            }
        };

        Self {
            event_type,
            timestamp: event.time,
        }
    }

    pub fn as_raw(&self) -> *const clap_event {
        // FIXME
        todo!()
    }

    pub fn into_raw(self) -> Option<clap_event> {
        use clap_sys::events::*;

        let (type_, data) = match self.event_type {
            EventType::NoteOn(note) => Some((
                CLAP_EVENT_NOTE_ON,
                clap_event_data {
                    note: note.into_raw(),
                },
            )),
            _ => None, // TODO
        }?;

        Some(clap_event {
            time: self.timestamp,
            type_,
            data,
        })
    }
}
