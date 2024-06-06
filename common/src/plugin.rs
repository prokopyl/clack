//! Utilities to implement or interact with plugins.

/// A set of standard plugin features meant to be used for a plugin descriptor's features.
///
/// Non-standard features should be formatted as: "$namespace:$feature"
pub mod features {
    use clap_sys::plugin_features::*;
    use std::ffi::CStr;

    /// `"instrument"`: The plugin can process note events and then produce audio
    pub const INSTRUMENT: &CStr = CLAP_PLUGIN_FEATURE_INSTRUMENT;
    /// `"audio-effect"`: The plugin is an audio effect
    pub const AUDIO_EFFECT: &CStr = CLAP_PLUGIN_FEATURE_AUDIO_EFFECT;
    /// `"note-effect"`: The plugin is a note effect or a note generator/sequencer
    pub const NOTE_EFFECT: &CStr = CLAP_PLUGIN_FEATURE_NOTE_EFFECT;
    /// `"analyzer"`: The plugin is an analyzer
    pub const ANALYZER: &CStr = CLAP_PLUGIN_FEATURE_ANALYZER;

    /// `"synthesizer"`
    pub const SYNTHESIZER: &CStr = CLAP_PLUGIN_FEATURE_SYNTHESIZER;
    /// `"sampler"`
    pub const SAMPLER: &CStr = CLAP_PLUGIN_FEATURE_SAMPLER;
    /// `"drum"`
    pub const DRUM: &CStr = CLAP_PLUGIN_FEATURE_DRUM;
    /// `"drum-machine"`
    pub const DRUM_MACHINE: &CStr = CLAP_PLUGIN_FEATURE_DRUM_MACHINE;

    /// `"filter"`
    pub const FILTER: &CStr = CLAP_PLUGIN_FEATURE_FILTER;
    /// `"phaser"`
    pub const PHASER: &CStr = CLAP_PLUGIN_FEATURE_PHASER;
    /// `"equalizer"`
    pub const EQUALIZER: &CStr = CLAP_PLUGIN_FEATURE_EQUALIZER;
    /// `"de-esser"`
    pub const DEESSER: &CStr = CLAP_PLUGIN_FEATURE_DEESSER;
    /// `"phase-vocoder"`
    pub const PHASE_VOCODER: &CStr = CLAP_PLUGIN_FEATURE_PHASE_VOCODER;
    /// `"granular"`
    pub const GRANULAR: &CStr = CLAP_PLUGIN_FEATURE_GRANULAR;
    /// `"frequency-shifter"`
    pub const FREQUENCY_SHIFTER: &CStr = CLAP_PLUGIN_FEATURE_FREQUENCY_SHIFTER;
    /// `"pitch-shifter"`
    pub const PITCH_SHIFTER: &CStr = CLAP_PLUGIN_FEATURE_PITCH_SHIFTER;

    /// `"distortion"`
    pub const DISTORTION: &CStr = CLAP_PLUGIN_FEATURE_DISTORTION;
    /// `"transient-shaper"`
    pub const TRANSIENT_SHAPER: &CStr = CLAP_PLUGIN_FEATURE_TRANSIENT_SHAPER;
    /// `"compressor"`
    pub const COMPRESSOR: &CStr = CLAP_PLUGIN_FEATURE_COMPRESSOR;
    /// `"limiter"`
    pub const LIMITER: &CStr = CLAP_PLUGIN_FEATURE_LIMITER;

    /// `"flanger"`
    pub const FLANGER: &CStr = CLAP_PLUGIN_FEATURE_FLANGER;
    /// `"chorus"`
    pub const CHORUS: &CStr = CLAP_PLUGIN_FEATURE_CHORUS;
    /// `"delay"`
    pub const DELAY: &CStr = CLAP_PLUGIN_FEATURE_DELAY;
    /// `"reverb"`
    pub const REVERB: &CStr = CLAP_PLUGIN_FEATURE_REVERB;

    /// `"tremolo"`
    pub const TREMOLO: &CStr = CLAP_PLUGIN_FEATURE_TREMOLO;
    /// `"glitch"`
    pub const GLITCH: &CStr = CLAP_PLUGIN_FEATURE_GLITCH;

    /// `"utility"`
    pub const UTILITY: &CStr = CLAP_PLUGIN_FEATURE_UTILITY;
    /// `"pitch-correction"`
    pub const PITCH_CORRECTION: &CStr = CLAP_PLUGIN_FEATURE_PITCH_CORRECTION;
    /// `"restoration"`
    pub const RESTORATION: &CStr = CLAP_PLUGIN_FEATURE_RESTORATION;

    /// `"multi-effects"`
    pub const MULTI_EFFECTS: &CStr = CLAP_PLUGIN_FEATURE_MULTI_EFFECTS;

    /// `"mixing"`
    pub const MIXING: &CStr = CLAP_PLUGIN_FEATURE_MIXING;
    /// `"mastering"`
    pub const MASTERING: &CStr = CLAP_PLUGIN_FEATURE_MASTERING;

    /// `"mono"`
    pub const MONO: &CStr = CLAP_PLUGIN_FEATURE_MONO;
    /// `"stereo"`
    pub const STEREO: &CStr = CLAP_PLUGIN_FEATURE_STEREO;
    /// `"surround"`
    pub const SURROUND: &CStr = CLAP_PLUGIN_FEATURE_SURROUND;
    /// `"ambisonic"`
    pub const AMBISONIC: &CStr = CLAP_PLUGIN_FEATURE_AMBISONIC;
}
