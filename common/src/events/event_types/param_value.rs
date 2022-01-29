use crate::events::EventHeader;
use bitflags::bitflags;
use clap_sys::events::{
    clap_event_param_mod, clap_event_param_value, CLAP_EVENT_BEGIN_ADJUST, CLAP_EVENT_END_ADJUST,
    CLAP_EVENT_IS_LIVE, CLAP_EVENT_SHOULD_RECORD,
};
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};

bitflags! {
    #[repr(C)]
    pub struct ParamEventFlags: u32 {
        const IS_LIVE = CLAP_EVENT_IS_LIVE;
        const BEGIN_ADJUST = CLAP_EVENT_BEGIN_ADJUST;
        const END_ADJUST = CLAP_EVENT_END_ADJUST;
        const SHOULD_RECORD = CLAP_EVENT_SHOULD_RECORD;
    }
}

pub struct ParamValueEvent {
    inner: clap_event_param_value,
}

impl ParamValueEvent {
    #[inline]
    pub fn new(
        header: EventHeader<Self>,
        cookie: *mut c_void,
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
            && self.inner.header == other.inner.header
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

pub struct ParamModEvent {
    inner: clap_event_param_mod,
}

impl ParamModEvent {
    #[inline]
    pub fn new(
        header: EventHeader<Self>,
        cookie: *mut c_void,
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
            && self.inner.header == other.inner.header
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
