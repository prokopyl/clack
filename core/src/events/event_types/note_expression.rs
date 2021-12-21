use crate::events::event_match::NoteEventMatch;
use clap_sys::events::*;

#[non_exhaustive]
#[repr(i32)]
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

pub struct NoteExpressionEvent {
    inner: clap_event_note_expression,
}

impl NoteExpressionEvent {
    #[inline]
    pub fn expression_type(&self) -> NoteExpressionType {
        NoteExpressionType::from_raw(self.inner.expression_id)
    }

    #[inline]
    pub fn port_index(&self) -> NoteEventMatch<i32> {
        NoteEventMatch::from_raw(self.inner.port_index)
    }

    #[inline]
    pub fn key(&self) -> NoteEventMatch<u8> {
        NoteEventMatch::from_raw(self.inner.key)
    }

    #[inline]
    pub fn channel(&self) -> NoteEventMatch<u8> {
        NoteEventMatch::from_raw(self.inner.channel)
    }

    #[inline]
    pub fn value(&self) -> f64 {
        self.inner.value
    }
}
