use crate::events::helpers::impl_event_helpers;
use crate::events::spaces::CoreEventSpace;
use crate::events::{impl_event_pckn, Event, EventFlags, EventHeader, Match, Pckn, UnknownEvent};
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
    pub const fn from_raw(raw: clap_note_expression) -> Option<Self> {
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
    pub const fn into_raw(self) -> clap_note_expression {
        self as clap_note_expression
    }
}

#[derive(Copy, Clone)]
pub struct NoteExpressionEvent {
    inner: clap_event_note_expression,
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for NoteExpressionEvent {
    const TYPE_ID: u16 = CLAP_EVENT_NOTE_EXPRESSION;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

impl AsRef<UnknownEvent> for NoteExpressionEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self.as_unknown()
    }
}

impl NoteExpressionEvent {
    #[inline]
    pub const fn new(
        time: u32,
        pckn: Pckn,
        expression_type: NoteExpressionType,
        value: f64,
    ) -> Self {
        Self {
            inner: clap_event_note_expression {
                header: EventHeader::<Self>::new_core(time, EventFlags::empty()).into_raw(),
                note_id: pckn.raw_note_id(),
                port_index: pckn.raw_port_index(),
                key: pckn.raw_key(),
                channel: pckn.raw_channel(),
                expression_id: expression_type.into_raw(),
                value,
            },
        }
    }

    #[inline]
    pub const fn expression_type(&self) -> Option<NoteExpressionType> {
        NoteExpressionType::from_raw(self.inner.expression_id)
    }

    #[inline]
    pub const fn set_expression_type(&mut self, expression_type: NoteExpressionType) {
        self.inner.expression_id = expression_type.into_raw()
    }

    #[inline]
    pub const fn with_expression_type(mut self, expression_type: NoteExpressionType) -> Self {
        self.inner.expression_id = expression_type.into_raw();
        self
    }

    #[inline]
    pub const fn value(&self) -> f64 {
        self.inner.value
    }

    #[inline]
    pub const fn set_value(&mut self, value: f64) {
        self.inner.value = value
    }

    #[inline]
    pub const fn with_value(mut self, value: f64) -> Self {
        self.inner.value = value;
        self
    }

    impl_event_helpers!(clap_event_note_expression);
    impl_event_pckn!();
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
