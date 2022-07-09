use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, EventHeader};
use bitflags::bitflags;
use clap_sys::events::{
    clap_event_param_gesture, clap_event_param_mod, clap_event_param_value, CLAP_EVENT_DONT_RECORD,
    CLAP_EVENT_IS_LIVE, CLAP_EVENT_PARAM_GESTURE_BEGIN, CLAP_EVENT_PARAM_GESTURE_END,
    CLAP_EVENT_PARAM_MOD, CLAP_EVENT_PARAM_VALUE,
};
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};

bitflags! {
    #[repr(C)]
    pub struct ParamEventFlags: u32 {
        const IS_LIVE = CLAP_EVENT_IS_LIVE;
        const BEGIN_ADJUST = CLAP_EVENT_PARAM_GESTURE_BEGIN as u32;
        const END_ADJUST = CLAP_EVENT_PARAM_GESTURE_END as u32;
        const DONT_RECORD = CLAP_EVENT_DONT_RECORD;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParamValueEvent {
    inner: clap_event_param_value,
}

unsafe impl<'a> Event<'a> for ParamValueEvent {
    const TYPE_ID: u16 = CLAP_EVENT_PARAM_VALUE as u16;
    type EventSpace = CoreEventSpace<'a>;
}

impl ParamValueEvent {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn new(
        header: EventHeader<Self>,
        cookie: *mut c_void,
        note_id: i32,
        param_id: u32,
        port_index: i16,
        channel: i16,
        key: i16,
        value: f64,
    ) -> Self {
        Self {
            inner: clap_event_param_value {
                header: header.into_raw(),
                cookie,
                note_id,
                param_id,
                port_index,
                key,
                channel,
                value,
            },
        }
    }

    #[inline]
    pub fn cookie(&self) -> *const c_void {
        self.inner.cookie
    }

    #[inline]
    pub fn param_id(&self) -> u32 {
        self.inner.param_id
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

    #[inline]
    pub fn from_raw(raw: clap_event_param_value) -> Self {
        Self { inner: raw }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_param_value {
        self.inner
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
    }
}

impl Debug for ParamValueEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParamValueEvent")
            .field("port_index", &self.inner.port_index)
            .field("channel", &self.inner.channel)
            .field("key", &self.inner.key)
            .field("param_id", &self.inner.param_id)
            .field("value", &self.inner.value)
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct ParamModEvent {
    inner: clap_event_param_mod,
}

unsafe impl<'a> Event<'a> for ParamModEvent {
    const TYPE_ID: u16 = CLAP_EVENT_PARAM_MOD as u16;
    type EventSpace = CoreEventSpace<'a>;
}

impl ParamModEvent {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        header: EventHeader<Self>,
        cookie: *mut c_void,
        note_id: i32,
        param_id: u32,
        port_index: i16,
        channel: i16,
        key: i16,
        amount: f64,
    ) -> Self {
        Self {
            inner: clap_event_param_mod {
                header: header.into_raw(),
                cookie,
                note_id,
                param_id,
                port_index,
                key,
                channel,
                amount,
            },
        }
    }

    #[inline]
    pub fn cookie(&self) -> *const c_void {
        self.inner.cookie
    }

    #[inline]
    pub fn param_id(&self) -> u32 {
        self.inner.param_id
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
        self.inner.amount
    }

    #[inline]
    pub fn from_raw(raw: clap_event_param_mod) -> Self {
        Self { inner: raw }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_param_mod {
        self.inner
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
    }
}

impl Debug for ParamModEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParamModEvent")
            .field("port_index", &self.inner.port_index)
            .field("channel", &self.inner.channel)
            .field("key", &self.inner.key)
            .field("param_id", &self.inner.param_id)
            .field("amount", &self.inner.amount)
            .finish()
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParamGestureBeginEvent {
    inner: clap_event_param_gesture,
}

unsafe impl<'a> Event<'a> for ParamGestureBeginEvent {
    const TYPE_ID: u16 = CLAP_EVENT_PARAM_GESTURE_BEGIN as u16;
    type EventSpace = CoreEventSpace<'a>;
}

impl ParamGestureBeginEvent {
    #[inline]
    pub const fn new(header: EventHeader<Self>, param_id: u32) -> Self {
        Self {
            inner: clap_event_param_gesture {
                header: header.into_raw(),
                param_id,
            },
        }
    }

    #[inline]
    pub const fn param_id(&self) -> u32 {
        self.inner.param_id
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ParamGestureEndEvent {
    inner: clap_event_param_gesture,
}

unsafe impl<'a> Event<'a> for ParamGestureEndEvent {
    const TYPE_ID: u16 = CLAP_EVENT_PARAM_GESTURE_END as u16;
    type EventSpace = CoreEventSpace<'a>;
}

impl ParamGestureEndEvent {
    #[inline]
    pub const fn new(header: EventHeader<Self>, param_id: u32) -> Self {
        Self {
            inner: clap_event_param_gesture {
                header: header.into_raw(),
                param_id,
            },
        }
    }

    #[inline]
    pub const fn param_id(&self) -> u32 {
        self.inner.param_id
    }
}
