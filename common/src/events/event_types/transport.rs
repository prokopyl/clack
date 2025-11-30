use crate::events::spaces::CoreEventSpace;
use crate::events::{Event, EventHeader, UnknownEvent};
use crate::utils::{BeatTime, SecondsTime};
use bitflags::bitflags;
use clap_sys::events::{
    CLAP_EVENT_TRANSPORT, CLAP_TRANSPORT_HAS_BEATS_TIMELINE, CLAP_TRANSPORT_HAS_SECONDS_TIMELINE,
    CLAP_TRANSPORT_HAS_TEMPO, CLAP_TRANSPORT_HAS_TIME_SIGNATURE, CLAP_TRANSPORT_IS_LOOP_ACTIVE,
    CLAP_TRANSPORT_IS_PLAYING, CLAP_TRANSPORT_IS_RECORDING, CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL,
    clap_event_transport,
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

    pub time_signature_numerator: u16,
    pub time_signature_denominator: u16,
}

// SAFETY: this matches the type ID and event space
unsafe impl Event for TransportEvent {
    const TYPE_ID: u16 = CLAP_EVENT_TRANSPORT;
    type EventSpace<'a> = CoreEventSpace<'a>;
}

impl AsRef<UnknownEvent> for TransportEvent {
    #[inline]
    fn as_ref(&self) -> &UnknownEvent {
        self.as_unknown()
    }
}

impl TransportEvent {
    #[inline]
    pub const fn as_raw(&self) -> &clap_event_transport {
        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        unsafe { &*(self as *const Self as *const clap_event_transport) }
    }

    #[inline]
    pub fn as_raw_mut(&mut self) -> &mut clap_event_transport {
        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        unsafe { &mut *(self as *mut Self as *mut clap_event_transport) }
    }

    #[inline]
    pub const fn from_raw(raw: &clap_event_transport) -> Self {
        crate::events::ensure_event_matches::<Self>(&raw.header);

        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        let this = unsafe { &*(raw as *const clap_event_transport as *const Self) };
        *this
    }

    #[inline]
    pub const fn from_raw_ref(raw: &clap_event_transport) -> &Self {
        crate::events::ensure_event_matches::<Self>(&raw.header);

        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        unsafe { &*(raw as *const clap_event_transport as *const Self) }
    }

    #[inline]
    pub const fn from_raw_mut(raw: &mut clap_event_transport) -> &mut Self {
        crate::events::ensure_event_matches::<Self>(&raw.header);

        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        unsafe { &mut *(raw as *mut clap_event_transport as *mut Self) }
    }
}
