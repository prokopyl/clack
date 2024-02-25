use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, EventHeader, UnknownEvent};
use crate::utils::{BeatTime, SecondsTime};
use bitflags::bitflags;
use clap_sys::events::{
    clap_event_transport, CLAP_EVENT_TRANSPORT, CLAP_TRANSPORT_HAS_BEATS_TIMELINE,
    CLAP_TRANSPORT_HAS_SECONDS_TIMELINE, CLAP_TRANSPORT_HAS_TEMPO,
    CLAP_TRANSPORT_HAS_TIME_SIGNATURE, CLAP_TRANSPORT_IS_LOOP_ACTIVE, CLAP_TRANSPORT_IS_PLAYING,
    CLAP_TRANSPORT_IS_RECORDING, CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL,
};

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TransportFlags: u32 {
        const HAS_TEMPO = CLAP_TRANSPORT_HAS_TEMPO;
        const HAS_BEATS_TIMELINE = CLAP_TRANSPORT_HAS_BEATS_TIMELINE;
        const HAS_SECONDS_TIMELINE = CLAP_TRANSPORT_HAS_SECONDS_TIMELINE;
        const HAS_TIME_SIGNATURE = CLAP_TRANSPORT_HAS_TIME_SIGNATURE;
        const IS_PLAYING = CLAP_TRANSPORT_IS_PLAYING;
        const IS_RECORDING = CLAP_TRANSPORT_IS_RECORDING;
        const IS_LOOP_ACTIVE = CLAP_TRANSPORT_IS_LOOP_ACTIVE;
        const IS_WITHIN_PRE_ROLL = CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL;
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct TransportEvent {
    pub header: EventHeader<TransportEvent>,

    pub flags: TransportFlags,

    pub song_pos_beats: BeatTime,
    pub song_pos_seconds: SecondsTime,

    pub tempo: f64,
    pub tempo_inc: f64,

    pub loop_start_beats: BeatTime,
    pub loop_end_beats: BeatTime,
    pub loop_start_seconds: SecondsTime,
    pub loop_end_seconds: SecondsTime,

    pub bar_start: BeatTime,
    pub bar_number: i32,

    pub time_signature_numerator: i16,
    pub time_signature_denominator: i16,
}

// SAFETY: this matches the type ID and event space
unsafe impl<'a> Event<'a> for TransportEvent {
    const TYPE_ID: u16 = CLAP_EVENT_TRANSPORT;
    type EventSpace = CoreEventSpace<'a>;
}

impl<'a> AsRef<UnknownEvent<'a>> for TransportEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent<'a> {
        self.as_unknown()
    }
}

impl TransportEvent {
    #[inline]
    pub fn from_raw(raw: clap_event_transport) -> Self {
        // SAFETY: TransportEvent is repr(C) and has the same memory representation
        unsafe { core::mem::transmute(raw) }
    }

    #[inline]
    pub fn from_raw_ref(raw: &clap_event_transport) -> &Self {
        // SAFETY: TransportEvent is repr(C) and has the same memory representation
        unsafe { core::mem::transmute(raw) }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_transport {
        // SAFETY: TransportEvent is repr(C) and has the same memory representation
        unsafe { core::mem::transmute(self) }
    }

    #[inline]
    pub fn as_raw_ref(&self) -> &clap_event_transport {
        // SAFETY: TransportEvent is repr(C) and has the same memory representation
        unsafe { core::mem::transmute(self) }
    }
}
