use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, EventHeader, UnknownEvent};
use clap_sys::events::*;
use std::fmt::{Debug, Formatter};

#[non_exhaustive]
#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NoteExpressionType {
    Volume = CLAP_NOTE_EXPRESSION_VOLUME,
    Pan = CLAP_NOTE_EXPRESSION_PAN,
    Tuning = CLAP_NOTE_EXPRESSION_TUNING,
    Vibrato = CLAP_NOTE_EXPRESSION_VIBRATO,
    Expression = CLAP_NOTE_EXPRESSION_EXPRESSION,
    Brightness = CLAP_NOTE_EXPRESSION_BRIGHTNESS,
    Pressure = CLAP_NOTE_EXPRESSION_PRESSURE,
}

impl NoteExpressionType {
    #[inline]
    pub fn from_raw(raw: clap_note_expression) -> Option<Self> {
        use NoteExpressionType::*;
        match raw {
            CLAP_NOTE_EXPRESSION_VOLUME => Some(Volume),
            CLAP_NOTE_EXPRESSION_PAN => Some(Pan),
            CLAP_NOTE_EXPRESSION_TUNING => Some(Tuning),
            CLAP_NOTE_EXPRESSION_VIBRATO => Some(Vibrato),
            CLAP_NOTE_EXPRESSION_BRIGHTNESS => Some(Brightness),
            CLAP_NOTE_EXPRESSION_PRESSURE => Some(Pressure),
            _ => None,
        }
    }

    #[inline]
    pub fn into_raw(self) -> clap_note_expression {
        self as clap_note_expression
    }
}

#[derive(Copy, Clone)]
pub struct NoteExpressionEvent {
    inner: clap_event_note_expression,
}

// SAFETY: this matches the type ID and event space
unsafe impl<'a> Event<'a> for NoteExpressionEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_EXPRESSION;
    type EventSpace = CoreEventSpace<'a>;
}

impl<'a> AsRef<UnknownEvent<'a>> for NoteExpressionEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent<'a> {
        self.as_unknown()
    }
}

impl NoteExpressionEvent {
    pub fn new(
        header: EventHeader<Self>,
        note_id: i32,
        port_index: i16,
        key: i16,
        channel: i16,
        value: f64,
        expression_type: NoteExpressionType,
    ) -> Self {
        Self {
            inner: clap_event_note_expression {
                header: header.into_raw(),
                note_id,
                port_index,
                key,
                channel,
                expression_id: expression_type.into_raw(),
                value,
            },
        }
    }
    #[inline]
    pub fn expression_type(&self) -> Option<NoteExpressionType> {
        NoteExpressionType::from_raw(self.inner.expression_id)
    }

    #[inline]
    pub fn note_id(&self) -> i32 {
        self.inner.note_id
    }

    #[inline]
    pub fn port_index(&self) -> i16 {
        self.inner.port_index
    }

    #[inline]
    pub fn set_port_index(&mut self, port_index: i16) {
        self.inner.port_index = port_index;
    }

    #[inline]
    pub fn key(&self) -> i16 {
        self.inner.key
    }

    #[inline]
    pub fn channel(&self) -> i16 {
        self.inner.channel
    }

    #[inline]
    pub fn value(&self) -> f64 {
        self.inner.value
    }
}

impl PartialEq for NoteExpressionEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.key == other.inner.key
            && self.inner.expression_id == other.inner.expression_id
            && self.inner.note_id == other.inner.note_id
            && self.inner.channel == other.inner.channel
            && self.inner.port_index == other.inner.port_index
            && self.inner.value == other.inner.value
    }
}

impl Debug for NoteExpressionEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoteExpressionEvent")
            .field("port_index", &self.inner.port_index)
            .field("channel", &self.inner.channel)
            .field("key", &self.inner.key)
            .field("expression_id", &self.inner.expression_id)
            .field("note_id", &self.inner.note_id)
            .field("value", &self.inner.value)
            .finish()
    }
}
