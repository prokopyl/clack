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
    /// A set of flags about the state of the transport in a [`TransportEvent`].
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TransportFlags: u32 {
        /// Whether the transport has a tempo at all.
        const HAS_TEMPO = CLAP_TRANSPORT_HAS_TEMPO;
        /// Whether the transport has a timeline in beats.
        const HAS_BEATS_TIMELINE = CLAP_TRANSPORT_HAS_BEATS_TIMELINE;
        /// Whether the transport has a timeline in seconds.
        const HAS_SECONDS_TIMELINE = CLAP_TRANSPORT_HAS_SECONDS_TIMELINE;
        /// Whether the transport has a time signature.
        const HAS_TIME_SIGNATURE = CLAP_TRANSPORT_HAS_TIME_SIGNATURE;
        /// Whether the transport is playing.
        const IS_PLAYING = CLAP_TRANSPORT_IS_PLAYING;
        /// Whether the transport is recording.
        const IS_RECORDING = CLAP_TRANSPORT_IS_RECORDING;
        /// Whether the transport's loop mode is active.
        const IS_LOOP_ACTIVE = CLAP_TRANSPORT_IS_LOOP_ACTIVE;
        /// Whether the transport is within its recording pre-roll.
        const IS_WITHIN_PRE_ROLL = CLAP_TRANSPORT_IS_WITHIN_PRE_ROLL;
    }
}

/// Informs the plugin that Host Transport event has changed.
///
/// This event is also directly given to the plugin's `process` function for every block.
#[repr(C)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct TransportEvent {
    /// The event's header.
    pub header: EventHeader<TransportEvent>,

    /// A set of boolean flags about the current state of the transport.
    pub flags: TransportFlags,

    /// The current song position, in beats.
    pub song_pos_beats: BeatTime,
    /// The current song position, in seconds.
    pub song_pos_seconds: SecondsTime,

    /// The current tempo, in Beats Per Minute.
    pub tempo: f64,
    /// The current tempo increment, in Beats Per Minute per sample.
    ///
    /// This tempo increment is valid until the next [`TransportEvent`] is received.
    pub tempo_inc: f64,

    /// The start position of the loop, in Beats.
    pub loop_start_beats: BeatTime,
    /// The end position of the loop, in Beats.
    pub loop_end_beats: BeatTime,
    /// The start position of the loop, in seconds.
    pub loop_start_seconds: SecondsTime,
    /// The end position of the loop, in seconds.
    pub loop_end_seconds: SecondsTime,

    /// The start position of the current bar, in beats.
    pub bar_start: BeatTime,
    /// The number of the current bar.
    ///
    /// This starts at `0`, e.g. the bar at song position `0` is `0`.
    pub bar_number: i32,

    /// The numerator of the current time signature.
    pub time_signature_numerator: u16,
    /// The denominator of the current time signature.
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
    /// Returns a shared reference to the underlying raw, C-FFI compatible event struct.
    #[inline]
    pub const fn as_raw(&self) -> &clap_event_transport {
        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        unsafe { &*(self as *const Self as *const clap_event_transport) }
    }

    /// Returns an exclusive, mutable reference to the raw, C-FFI compatible event struct.
    ///
    /// # Safety
    ///
    /// This method allows mutating the event header's fields, most notably `type_`, `size` and
    /// `space_id`.
    ///
    /// Modifying those fields may trigger Undefined Behavior, as it can make other readers of this
    /// event treat it as a type or size that it is actually not.
    #[inline]
    pub const unsafe fn as_raw_mut(&mut self) -> &mut clap_event_transport {
        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        unsafe { &mut *(self as *mut Self as *mut clap_event_transport) }
    }

    /// Creates a new event of this type from a reference to a raw, C-FFI compatible event
    /// struct.
    ///
    /// # Panics
    ///
    /// This method will panic if the given event struct's header doesn't actually match
    /// the expected event type.
    #[inline]
    pub const fn from_raw(raw: &clap_event_transport) -> Self {
        crate::events::ensure_event_matches::<Self>(&raw.header);

        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        let this = unsafe { &*(raw as *const clap_event_transport as *const Self) };
        *this
    }

    /// Creates a reference to an event of this type from a reference to a raw,
    /// C-FFI compatible event struct.
    ///
    /// # Panics
    ///
    /// This method will panic if the given event struct's header doesn't actually match
    /// the expected event type.
    #[inline]
    pub const fn from_raw_ref(raw: &clap_event_transport) -> &Self {
        crate::events::ensure_event_matches::<Self>(&raw.header);

        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        unsafe { &*(raw as *const clap_event_transport as *const Self) }
    }

    /// Creates a mutable reference to a note event of this type from a reference to a raw,
    /// C-FFI compatible event struct.
    ///
    /// # Panics
    ///
    /// This method will panic if the given event struct's header doesn't actually match
    /// the expected note event type.
    #[inline]
    pub const fn from_raw_mut(raw: &mut clap_event_transport) -> &mut Self {
        crate::events::ensure_event_matches::<Self>(&raw.header);

        // SAFETY: This type is #[repr(C)]-compatible with clap_event_transport
        unsafe { &mut *(raw as *mut clap_event_transport as *mut Self) }
    }
}
