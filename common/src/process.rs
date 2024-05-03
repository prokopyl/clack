use clap_sys::process::*;
use std::fmt::Debug;

mod constant_mask;
pub use constant_mask::*;

/// Status returned by a plugin after processing.
///
/// This is mainly used as a way for the plugin to tell the host when it can be safely put to sleep.
///
/// Note that Clack uses a [`Result`] enum for relaying a failed processing to the host,
/// unlike the C CLAP API which uses an extra state in enum (`CLAP_PROCESS_ERROR`) to indicate failure.
#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProcessStatus {
    /// Processing should continue: the plugin has no desire to be put to sleep.
    Continue = CLAP_PROCESS_CONTINUE,
    /// Processing should continue, unless all outputs are quiet.
    ContinueIfNotQuiet = CLAP_PROCESS_CONTINUE_IF_NOT_QUIET,
    /// The plugin is currently processing its tail (e.g. release, reverb, etc.).
    ///
    /// Use the `tail` extension to query the plugin for its current tail length.
    Tail = CLAP_PROCESS_TAIL,
    /// No more processing is required until the next event or variation in audio input.
    Sleep = CLAP_PROCESS_SLEEP,
}

impl ProcessStatus {
    /// Gets a [`ProcessStatus`] from the raw, C-FFI compatible value.
    ///
    /// In order to match Clack's APIs, this returns `Some(Err(()))` if the value is
    /// `CLAP_PROCESS_ERROR`.
    ///
    /// If the given integer does not match any known CLAP Processing status codes, [`None`] is
    /// returned.
    pub fn from_raw(raw: clap_process_status) -> Option<Result<Self, ()>> {
        use ProcessStatus::*;

        match raw {
            CLAP_PROCESS_CONTINUE => Some(Ok(Continue)),
            CLAP_PROCESS_CONTINUE_IF_NOT_QUIET => Some(Ok(ContinueIfNotQuiet)),
            CLAP_PROCESS_SLEEP => Some(Ok(Sleep)),
            CLAP_PROCESS_TAIL => Some(Ok(Tail)),
            CLAP_PROCESS_ERROR => Some(Err(())),
            _ => None,
        }
    }

    pub fn combined_with(self, other: ProcessStatus) -> ProcessStatus {
        use ProcessStatus::*;

        match (self, other) {
            (Continue, _) | (_, Continue) => Continue,
            (ContinueIfNotQuiet, _) | (_, ContinueIfNotQuiet) => ContinueIfNotQuiet,
            (Tail, _) | (_, Tail) => Tail,
            (Sleep, Sleep) => Sleep,
        }
    }
}

/// The audio configuration passed to a plugin's audio processor upon activation.
///
/// Those settings are constant throughout the audio processor's lifetime,
/// i.e. from a plugin's activation until its deactivation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PluginAudioConfiguration {
    /// The audio's sample rate.
    pub sample_rate: f64,
    /// The minimum amount of frames that will be processed at once.
    pub min_frames_count: u32,
    /// The maximum amount of frames that will be processed at once.
    pub max_frames_count: u32,
}

use clap_sys::audio_buffer::clap_audio_buffer;

pub struct AudioPortProcessingInfo {
    channel_count: u32,
    latency: u32,
    constant_mask: ConstantMask,
}

impl AudioPortProcessingInfo {
    #[inline]
    pub fn from_raw(raw: &clap_audio_buffer) -> Self {
        Self {
            channel_count: raw.channel_count,
            latency: raw.latency,
            constant_mask: ConstantMask::from_bits(raw.constant_mask),
        }
    }

    #[inline]
    pub fn channel_count(&self) -> u32 {
        self.channel_count
    }

    #[inline]
    pub fn latency(&self) -> u32 {
        self.latency
    }

    #[inline]
    pub fn constant_mask(&self) -> ConstantMask {
        self.constant_mask
    }
}
