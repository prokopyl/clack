use crate::events::event_match::EventTarget;
use clap_sys::events::*;
use std::fmt::{Debug, Formatter};

#[non_exhaustive]
#[repr(i32)]
#[derive(Copy, Clone)]
pub enum NoteExpressionType {
    Volume = CLAP_NOTE_EXPRESSION_VOLUME,
    Pan = CLAP_NOTE_EXPRESSION_PAN,
    Tuning = CLAP_NOTE_EXPRESSION_TUNING,
    Vibrato = CLAP_NOTE_EXPRESSION_VIBRATO,
    Brightness = CLAP_NOTE_EXPRESSION_BRIGHTNESS,
    Breath = CLAP_NOTE_EXPRESSION_BREATH,
    Pressure = CLAP_NOTE_EXPRESSION_PRESSURE,
    Timbre = CLAP_NOTE_EXPRESSION_TIMBRE,

    Unknown = i32::MAX,
}

impl NoteExpressionType {
    #[inline]
    pub fn from_raw(raw: i32) -> Self {
        use NoteExpressionType::*;
        match raw {
            CLAP_NOTE_EXPRESSION_VOLUME => Volume,
            CLAP_NOTE_EXPRESSION_PAN => Pan,
            CLAP_NOTE_EXPRESSION_TUNING => Tuning,
            CLAP_NOTE_EXPRESSION_VIBRATO => Vibrato,
            CLAP_NOTE_EXPRESSION_BRIGHTNESS => Brightness,
            CLAP_NOTE_EXPRESSION_BREATH => Breath,
            CLAP_NOTE_EXPRESSION_PRESSURE => Pressure,
            CLAP_NOTE_EXPRESSION_TIMBRE => Timbre,
            _ => Unknown,
        }
    }

    #[inline]
    pub fn into_raw(self) -> i32 {
        self as i32
    }
}

#[derive(Copy, Clone)]
pub struct NoteExpressionEvent {
    inner: clap_event_note_expression,
}

impl NoteExpressionEvent {
    #[inline]
    pub(crate) fn from_raw(inner: clap_event_note_expression) -> Self {
        Self { inner }
    }

    #[inline]
    pub(crate) fn into_raw(self) -> clap_event_note_expression {
        self.inner
    }

    #[inline]
    pub fn expression_type(&self) -> NoteExpressionType {
        NoteExpressionType::from_raw(self.inner.expression_id)
    }

    #[inline]
    pub fn port_index(&self) -> EventTarget<u16> {
        EventTarget::from_raw(self.inner.port_index)
    }

    #[inline]
    pub fn key(&self) -> EventTarget<u8> {
        EventTarget::from_raw(self.inner.key)
    }

    #[inline]
    pub fn channel(&self) -> EventTarget<u8> {
        EventTarget::from_raw(self.inner.channel)
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
            .field("value", &self.inner.value)
            .finish()
    }
}
