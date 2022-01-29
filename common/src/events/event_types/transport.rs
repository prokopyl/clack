use crate::events::EventHeader;
use bitflags::bitflags;
use clap_sys::events::{
    clap_event_transport, CLAP_TRANSPORT_HAS_BEATS_TIMELINE, CLAP_TRANSPORT_HAS_SECONDS_TIMELINE,
    CLAP_TRANSPORT_HAS_TEMPO, CLAP_TRANSPORT_HAS_TIME_SIGNATURE, CLAP_TRANSPORT_IS_LOOP_ACTIVE,
    CLAP_TRANSPORT_IS_PLAYING, CLAP_TRANSPORT_IS_RECORDING, CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL,
};

bitflags! {
    #[repr(C)]
    pub struct TransportEventFlags: u32 {
        const CLAP_TRANSPORT_HAS_TEMPO = CLAP_TRANSPORT_HAS_TEMPO;
        const CLAP_TRANSPORT_HAS_BEATS_TIMELINE = CLAP_TRANSPORT_HAS_BEATS_TIMELINE;
        const CLAP_TRANSPORT_HAS_SECONDS_TIMELINE = CLAP_TRANSPORT_HAS_SECONDS_TIMELINE;
        const CLAP_TRANSPORT_HAS_TIME_SIGNATURE = CLAP_TRANSPORT_HAS_TIME_SIGNATURE;
        const CLAP_TRANSPORT_IS_PLAYING = CLAP_TRANSPORT_IS_PLAYING;
        const CLAP_TRANSPORT_IS_RECORDING = CLAP_TRANSPORT_IS_RECORDING;
        const CLAP_TRANSPORT_IS_LOOP_ACTIVE = CLAP_TRANSPORT_IS_LOOP_ACTIVE;
        const CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL = CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL;
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct TransportEvent {
    pub header: EventHeader<TransportEvent>,
    pub song_pos_beats: i64,
    pub song_pos_seconds: i64,
    pub tempo: f64,
    pub tempo_inc: f64,
    pub bar_start: i64,
    pub bar_number: i32,
    pub loop_start_beats: i64,
    pub loop_end_beats: i64,
    pub loop_start_seconds: i64,
    pub loop_end_seconds: i64,
    pub time_signature_numerator: i16,
    pub time_signature_denominator: i16,
}

impl TransportEvent {
    pub fn from_raw(raw: clap_event_transport) -> Self {
        // SAFETY: TransportEvent is repr(C) and has the same memory representation
        unsafe { ::core::mem::transmute(raw) }
    }

    #[inline]
    pub fn into_raw(self) -> clap_event_transport {
        // SAFETY: TransportEvent is repr(C) and has the same memory representation
        unsafe { ::core::mem::transmute(self) }
    }
}
