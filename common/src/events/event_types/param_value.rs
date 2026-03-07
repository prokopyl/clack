use crate::events::helpers::impl_event_helpers;
use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, EventFlags, EventHeader, Match, Pckn, UnknownEvent, impl_event_pckn};
use crate::utils::{ClapId, Cookie};
use clap_sys::events::{
    CLAP_EVENT_PARAM_GESTURE_BEGIN, CLAP_EVENT_PARAM_GESTURE_END, CLAP_EVENT_PARAM_MOD,
    CLAP_EVENT_PARAM_VALUE, clap_event_param_gesture, clap_event_param_mod, clap_event_param_value,
};
use std::fmt::{Debug, Formatter};

/// Sets a parameter's value.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ParamValueEvent {
    inner: clap_event_param_value,
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for ParamValueEvent {
    const TYPE_ID: u16 = CLAP_EVENT_PARAM_VALUE;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

impl AsRef<UnknownEvent> for ParamValueEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self.as_unknown()
    }
}

impl ParamValueEvent {
    /// Creates a new [`ParamValueEvent`] from a `time` stamp, a parameter ID, a [`Pckn`] target,
    /// a parameter `value` and an optional [`Cookie`].
    #[inline]
    pub const fn new(time: u32, param_id: ClapId, pckn: Pckn, value: f64, cookie: Cookie) -> Self {
        Self {
            inner: clap_event_param_value {
                header: EventHeader::<Self>::new_core(time, EventFlags::empty()).into_raw(),
                param_id: param_id.get(),
                note_id: pckn.raw_note_id(),
                port_index: pckn.raw_port_index(),
                key: pckn.raw_key(),
                channel: pckn.raw_channel(),
                value,
                cookie: cookie.as_raw(),
            },
        }
    }

    /// Returns the parameter ID this event targets.
    ///
    /// This returns [`None`] if the parameter ID is invalid.
    #[inline]
    pub const fn param_id(&self) -> Option<ClapId> {
        ClapId::from_raw(self.inner.param_id)
    }

    /// Sets the parameter ID this event targets.
    #[inline]
    pub const fn set_param_id(&mut self, param_id: ClapId) {
        self.inner.param_id = param_id.get()
    }

    /// Builds a [`ParamValueEvent`] with the given parameter ID target, and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_param_id(mut self, param_id: ClapId) -> Self {
        self.inner.param_id = param_id.get();
        self
    }

    /// Returns the parameter value of this event.
    #[inline]
    pub const fn value(&self) -> f64 {
        self.inner.value
    }

    /// Sets the parameter value of this event.
    #[inline]
    pub const fn set_value(&mut self, value: f64) {
        self.inner.value = value
    }

    /// Builds a [`ParamValueEvent`] with the given parameter value, and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_value(mut self, value: f64) -> Self {
        self.inner.value = value;
        self
    }

    impl_event_helpers!(clap_event_param_value);
    impl_event_pckn!(self.inner);

    /// The [`Cookie`] sent alongside this event.
    #[inline]
    pub const fn cookie(&self) -> Cookie {
        Cookie::from_raw(self.inner.cookie)
    }

    /// Sets the given [`Cookie`] to be sent alongside this event.
    #[inline]
    pub const fn set_cookie(&mut self, cookie: Cookie) {
        self.inner.cookie = cookie.as_raw()
    }

    /// Builds a [`ParamValueEvent`] with the given [`Cookie`], and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_cookie(mut self, cookie: Cookie) -> Self {
        self.inner.cookie = cookie.as_raw();
        self
    }
}

impl PartialEq for ParamValueEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.key == other.inner.key
            && self.inner.header.time == other.inner.header.time
            && self.inner.channel == other.inner.channel
            && self.inner.port_index == other.inner.port_index
            && self.inner.param_id == other.inner.param_id
            && self.inner.value == other.inner.value
            && self.inner.note_id == other.inner.note_id
    }
}

impl Debug for ParamValueEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParamValueEvent")
            .field("header", &self.header())
            .field("port_index", &self.inner.port_index)
            .field("channel", &self.inner.channel)
            .field("key", &self.inner.key)
            .field("param_id", &self.inner.param_id)
            .field("note_id", &self.inner.note_id)
            .field("value", &self.inner.value)
            .finish()
    }
}

/// Sets a parameter modulation amount.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ParamModEvent {
    inner: clap_event_param_mod,
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for ParamModEvent {
    const TYPE_ID: u16 = CLAP_EVENT_PARAM_MOD;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

impl AsRef<UnknownEvent> for ParamModEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self.as_unknown()
    }
}

impl ParamModEvent {
    /// Creates a new [`ParamModEvent`] from a `time` stamp, a parameter ID, a [`Pckn`] target,
    /// a parameter modulation `amount` and an optional [`Cookie`].
    #[inline]
    pub const fn new(time: u32, param_id: ClapId, pckn: Pckn, amount: f64, cookie: Cookie) -> Self {
        Self {
            inner: clap_event_param_mod {
                header: EventHeader::<Self>::new_core(time, EventFlags::empty()).into_raw(),
                param_id: param_id.get(),
                note_id: pckn.raw_note_id(),
                port_index: pckn.raw_port_index(),
                key: pckn.raw_key(),
                channel: pckn.raw_channel(),
                amount,
                cookie: cookie.as_raw(),
            },
        }
    }

    /// Returns the parameter ID this event targets.
    ///
    /// This returns [`None`] if the parameter ID is invalid.
    #[inline]
    pub const fn param_id(&self) -> Option<ClapId> {
        ClapId::from_raw(self.inner.param_id)
    }

    /// Sets the parameter ID this event targets.
    #[inline]
    pub fn set_param_id(&mut self, param_id: ClapId) {
        self.inner.param_id = param_id.get()
    }

    /// Builds a [`ParamValueEvent`] with the given parameter ID target, and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_param_id(mut self, param_id: ClapId) -> Self {
        self.inner.param_id = param_id.get();
        self
    }

    /// Returns the parameter modulation amount of this event.
    #[inline]
    pub const fn amount(&self) -> f64 {
        self.inner.amount
    }

    /// Sets the parameter modulation amount of this event.
    #[inline]
    pub fn set_amount(&mut self, amount: f64) {
        self.inner.amount = amount
    }

    /// Builds a [`ParamModEvent`] with the given parameter modulation amount, and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_amount(mut self, amount: f64) -> Self {
        self.inner.amount = amount;
        self
    }

    impl_event_helpers!(clap_event_param_mod);
    impl_event_pckn!(self.inner);

    /// The [`Cookie`] sent alongside this event.
    #[inline]
    pub const fn cookie(&self) -> Cookie {
        Cookie::from_raw(self.inner.cookie)
    }

    /// Sets the given [`Cookie`] to be sent alongside this event.
    #[inline]
    pub fn set_cookie(&mut self, cookie: Cookie) {
        self.inner.cookie = cookie.as_raw()
    }

    /// Builds a [`ParamValueEvent`] with the given [`Cookie`], and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_cookie(mut self, cookie: Cookie) -> Self {
        self.inner.cookie = cookie.as_raw();
        self
    }
}

impl PartialEq for ParamModEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.key == other.inner.key
            && self.inner.header.time == other.inner.header.time
            && self.inner.channel == other.inner.channel
            && self.inner.port_index == other.inner.port_index
            && self.inner.param_id == other.inner.param_id
            && self.inner.amount == other.inner.amount
            && self.inner.note_id == other.inner.note_id
    }
}

impl Debug for ParamModEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParamModEvent")
            .field("header", &self.header())
            .field("port_index", &self.inner.port_index)
            .field("channel", &self.inner.channel)
            .field("key", &self.inner.key)
            .field("param_id", &self.inner.param_id)
            .field("note_id", &self.inner.note_id)
            .field("amount", &self.inner.amount)
            .finish()
    }
}

/// Indicates that the user started adjusting a knob or parameter value.
///
/// For the event to raise when the user finishes this adjustment, see [`ParamGestureEndEvent`].
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ParamGestureBeginEvent {
    inner: clap_event_param_gesture,
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for ParamGestureBeginEvent {
    const TYPE_ID: u16 = CLAP_EVENT_PARAM_GESTURE_BEGIN;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

impl AsRef<UnknownEvent> for ParamGestureBeginEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self.as_unknown()
    }
}

impl ParamGestureBeginEvent {
    /// Creates a new [`ParamGestureBeginEvent`] from a `time` stamp and a parameter ID.
    #[inline]
    pub const fn new(time: u32, param_id: ClapId) -> Self {
        Self {
            inner: clap_event_param_gesture {
                header: EventHeader::<Self>::new_core(time, EventFlags::empty()).into_raw(),
                param_id: param_id.get(),
            },
        }
    }

    /// Returns the parameter ID this event targets.
    ///
    /// This returns [`None`] if the parameter ID is invalid.
    #[inline]
    pub const fn param_id(&self) -> Option<ClapId> {
        ClapId::from_raw(self.inner.param_id)
    }

    /// Sets the parameter ID this event targets.
    #[inline]
    pub fn set_param_id(&mut self, param_id: ClapId) {
        self.inner.param_id = param_id.get()
    }

    /// Builds a [`ParamGestureBeginEvent`] with the given parameter ID target, and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_param_id(mut self, param_id: ClapId) -> Self {
        self.inner.param_id = param_id.get();
        self
    }
}

impl Debug for ParamGestureBeginEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParamGestureBeginEvent")
            .field("header", &self.header())
            .field("header", &self.header())
            .field("param_id", &self.inner.param_id)
            .finish()
    }
}

impl PartialEq for ParamGestureBeginEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.header.time == other.inner.header.time
            && self.inner.param_id == other.inner.param_id
    }
}

/// Indicates that the user finished adjusting a knob or parameter value.
///
/// For the event to raise when the user starts this adjustment, see [`ParamGestureBeginEvent`].
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ParamGestureEndEvent {
    inner: clap_event_param_gesture,
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for ParamGestureEndEvent {
    const TYPE_ID: u16 = CLAP_EVENT_PARAM_GESTURE_END;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

impl AsRef<UnknownEvent> for ParamGestureEndEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self.as_unknown()
    }
}

impl ParamGestureEndEvent {
    /// Creates a new [`ParamGestureEndEvent`] from a `time` stamp and a parameter ID.
    #[inline]
    pub const fn new(time: u32, param_id: ClapId) -> Self {
        Self {
            inner: clap_event_param_gesture {
                header: EventHeader::<Self>::new_core(time, EventFlags::empty()).into_raw(),
                param_id: param_id.get(),
            },
        }
    }

    /// Returns the parameter ID this event targets.
    ///
    /// This returns [`None`] if the parameter ID is invalid.
    #[inline]
    pub const fn param_id(&self) -> Option<ClapId> {
        ClapId::from_raw(self.inner.param_id)
    }

    /// Sets the parameter ID this event targets.
    #[inline]
    pub fn set_param_id(&mut self, param_id: ClapId) {
        self.inner.param_id = param_id.get()
    }

    /// Builds a [`ParamGestureEndEvent`] with the given parameter ID target, and returns it.
    ///
    /// This is useful to use in a builder-style pattern.
    #[inline]
    pub const fn with_param_id(mut self, param_id: ClapId) -> Self {
        self.inner.param_id = param_id.get();
        self
    }
}

impl Debug for ParamGestureEndEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParamGestureEndEvent")
            .field("header", &self.header())
            .field("header", self.header())
            .field("param_id", &self.inner.param_id)
            .finish()
    }
}

impl PartialEq for ParamGestureEndEvent {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.header.time == other.inner.header.time
            && self.inner.param_id == other.inner.param_id
    }
}
