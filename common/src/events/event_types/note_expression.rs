use crate::events::helpers::impl_event_helpers;
use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, EventFlags, EventHeader, Match, Pckn, UnknownEvent, impl_event_pckn};
use clap_sys::events::*;
use std::fmt::{Debug, Formatter};

/// The types of expressions a [`NoteExpressionEvent`] can carry.
#[non_exhaustive]
#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NoteExpressionType {
    /// A volume change.
    ///
    /// with 0 < x <= 4, plain = 20 * log(x)
    Volume = CLAP_NOTE_EXPRESSION_VOLUME,
    /// A panning change.
    ///
    /// 0 is left, 0.5 is center, and 1 is right.
    Pan = CLAP_NOTE_EXPRESSION_PAN,

    /// A tuning change.
    ///
    /// This is relative and is expressed in semitones, from -120 to +120.
    ///
    /// Semitones are in equal temperament; the resulting note would be
    /// retuned by `100 * event.value()` cents.
    Tuning = CLAP_NOTE_EXPRESSION_TUNING,
    /// A vibrato change.
    ///
    /// Values for this event type are in the `0..=1` range.
    Vibrato = CLAP_NOTE_EXPRESSION_VIBRATO,
    /// An "expression" change.
    ///
    /// Values for this event type are in the `0..=1` range.
    Expression = CLAP_NOTE_EXPRESSION_EXPRESSION,
    /// A "brightness" change.
    ///
    /// Values for this event type are in the `0..=1` range.
    Brightness = CLAP_NOTE_EXPRESSION_BRIGHTNESS,
    /// A "pressure" change.
    ///
    /// Values for this event type are in the `0..=1` range.
    Pressure = CLAP_NOTE_EXPRESSION_PRESSURE,
}

impl NoteExpressionType {
    /// Returns the [`NoteExpressionType`] matching the given raw, C-FFI compatible expression
    /// value.
    ///
    /// This returns `None` if the given value does not match any known [`NoteExpressionType`].
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

    /// Returns the raw, C-FFI compatible expression value corresponding to this [`NoteExpressionType`].
    #[inline]
    pub const fn into_raw(self) -> clap_note_expression {
        self as clap_note_expression
    }
}

/// A note expression event.
///
/// Note Expressions are well named modifications of a voice targeted to
/// voices using [`Pckn`s](Pckn). Note Expressions are delivered
/// as sample accurate events and should be applied at the sample when received.
///
/// Note expressions are a statement of value, not cumulative. A [`Pan`](NoteExpressionType::Pan) event of 0 followed by 1
/// followed by 0.5 would pan hard left, hard right, and center. They are intended as
/// an offset from the non-note-expression voice default. A voice which had a volume of
/// -20dB absent note expressions which received a +4dB note expression would move the
/// voice to -16dB.
///
/// A plugin which receives a note expression at the same sample as a [`NoteOnEvent`](super::NoteOnEvent)
/// should apply that expression to all generated samples. A plugin which receives
/// a note expression after a [`NoteOnEvent`](super::NoteOnEvent) event should initiate the voice with default
/// values and then apply the note expression when received. A plugin may make a choice
/// to smooth note expression streams.
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
    /// Creates a new [`NoteExpressionEvent`] from a `time` stamp, a [`Pckn`] target, an [`NoteExpressionType`] and
    /// a raw `value`.
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

    /// Returns the [`NoteExpressionType`] of this event.
    ///
    /// This returns `None` if the expression type is not supported.
    #[inline]
    pub const fn expression_type(&self) -> Option<NoteExpressionType> {
        NoteExpressionType::from_raw(self.inner.expression_id)
    }

    /// Sets the [`NoteExpressionType`] of this event.
    #[inline]
    pub const fn set_expression_type(&mut self, expression_type: NoteExpressionType) {
        self.inner.expression_id = expression_type.into_raw()
    }

    /// Builds a [`NoteExpressionEvent`] with the given [`NoteExpressionType`], and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_expression_type(mut self, expression_type: NoteExpressionType) -> Self {
        self.inner.expression_id = expression_type.into_raw();
        self
    }

    /// Returns the raw expression value of this event.
    #[inline]
    pub const fn value(&self) -> f64 {
        self.inner.value
    }

    /// Sets the raw expression value of this event.
    #[inline]
    pub const fn set_value(&mut self, value: f64) {
        self.inner.value = value
    }

    /// Builds a [`NoteExpressionEvent`] with the given raw expression value, and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_value(mut self, value: f64) -> Self {
        self.inner.value = value;
        self
    }

    impl_event_helpers!(clap_event_note_expression);
    impl_event_pckn!(self.inner);
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
