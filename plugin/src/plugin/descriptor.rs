use clap_sys::plugin::clap_plugin_descriptor;
use clap_sys::version::CLAP_VERSION;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Represents a type that can provide metadata about a given Plugin, such as its ID, name, version,
/// and more.
///
/// Note only the [`id`](PluginDescriptor::id) and [`name`](PluginDescriptor::name) fields are
/// mandatory, and should not be blank. All of the other fields are optional and can return [`None`].
///
/// See the documentation of each individual method to learn about the available metadata.
pub trait PluginDescriptor: 'static {
    /// The unique identifier of a plugin. This field is **mandatory**, and should not be blank.
    ///
    /// This identifier should be as globally-unique as possible to any users that might load this
    /// plugin, as this is the key hosts will use to differentiate between different plugins.
    ///
    /// Example: `com.u-he.diva`.
    fn id(&self) -> &CStr;

    /// The user-facing display name of this plugin. This field is **mandatory**, and should not be blank.
    ///
    /// This name will be displayed in plugin lists and selectors, and will be the main way users
    /// will find and differentiate the plugin.
    ///
    /// Example: `Diva`.
    fn name(&self) -> &CStr;

    /// The vendor of the plugin.
    ///
    /// Example: `u-he`.
    #[inline]
    fn vendor(&self) -> Option<&CStr> {
        None
    }

    /// The URL of this plugin's homepage.
    ///
    /// Example: `https://u-he.com/products/diva/`.
    #[inline]
    fn url(&self) -> Option<&CStr> {
        None
    }

    /// The URL of this plugin's user's manual.
    ///
    /// Example: `https://dl.u-he.com/manuals/plugins/diva/Diva-user-guide.pdf`.
    #[inline]
    fn manual_url(&self) -> Option<&CStr> {
        None
    }

    /// The URL of this plugin's support page.
    ///
    /// Example: `https://u-he.com/support/`.
    #[inline]
    fn support_url(&self) -> Option<&CStr> {
        None
    }

    /// The version of this plugin.
    ///
    /// While Semver-compatible version strings are preferred, this field can contain any arbitrary
    /// string.
    ///
    /// Example: `1.4.4`.
    #[inline]
    fn version(&self) -> Option<&CStr> {
        None
    }

    /// A short description of this plugin.
    ///
    /// Example: `The spirit of analogue`.
    #[inline]
    fn description(&self) -> Option<&CStr> {
        None
    }

    /// An arbitrary list of tags that can be used by hosts to classify this plugin.
    ///
    /// For some standard features, see the constants in the [`features`] module.
    ///
    /// Example: `"instrument", "synthesizer", "stereo"`.
    #[inline]
    #[allow(unused)]
    fn feature_at(&self, index: usize) -> Option<&CStr> {
        None
    }

    /// The number of features exposed by this descriptor.
    #[inline]
    fn features_count(&self) -> usize {
        0
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct StaticPluginDescriptor {
    pub id: &'static CStr,
    pub name: &'static CStr,
    pub vendor: Option<&'static CStr>,
    pub url: Option<&'static CStr>,
    pub manual_url: Option<&'static CStr>,
    pub support_url: Option<&'static CStr>,
    pub version: Option<&'static CStr>,
    pub description: Option<&'static CStr>,
    pub features: Option<&'static [&'static CStr]>,
}

impl PluginDescriptor for StaticPluginDescriptor {
    #[inline]
    fn id(&self) -> &CStr {
        self.id
    }

    #[inline]
    fn name(&self) -> &CStr {
        self.name
    }

    #[inline]
    fn vendor(&self) -> Option<&CStr> {
        self.vendor
    }

    #[inline]
    fn url(&self) -> Option<&CStr> {
        self.url
    }

    #[inline]
    fn manual_url(&self) -> Option<&CStr> {
        self.manual_url
    }

    #[inline]
    fn support_url(&self) -> Option<&CStr> {
        self.support_url
    }

    #[inline]
    fn version(&self) -> Option<&CStr> {
        self.version
    }

    #[inline]
    fn description(&self) -> Option<&CStr> {
        self.description
    }

    #[inline]
    fn feature_at(&self, index: usize) -> Option<&CStr> {
        self.features.and_then(|f| f.get(index).copied())
    }

    #[inline]
    fn features_count(&self) -> usize {
        self.features.map(|f| f.len()).unwrap_or(0)
    }
}

pub struct PluginDescriptorWrapper {
    descriptor: Box<dyn PluginDescriptor>,
    _features_array: Vec<*const c_char>,
    raw_descriptor: clap_plugin_descriptor,
}

// SAFETY: there is a null byte in this string.
const EMPTY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"\0") };

impl PluginDescriptorWrapper {
    pub fn new(descriptor: Box<dyn PluginDescriptor>) -> Self {
        let mut features_array: Vec<_> = (0..descriptor.features_count())
            .filter_map(|i| descriptor.feature_at(i))
            .map(|s| s.as_ptr())
            .collect();

        features_array.push(core::ptr::null());

        Self {
            raw_descriptor: clap_plugin_descriptor {
                clap_version: CLAP_VERSION,
                id: descriptor.id().as_ptr(),
                name: descriptor.name().as_ptr(),
                vendor: descriptor.vendor().unwrap_or(EMPTY).as_ptr(),
                url: descriptor.url().unwrap_or(EMPTY).as_ptr(),
                manual_url: descriptor.manual_url().unwrap_or(EMPTY).as_ptr(),
                support_url: descriptor.support_url().unwrap_or(EMPTY).as_ptr(),
                version: descriptor.version().unwrap_or(EMPTY).as_ptr(),
                description: descriptor.description().unwrap_or(EMPTY).as_ptr(),
                features: features_array.as_ptr(),
            },
            _features_array: features_array,
            descriptor,
        }
    }

    #[inline]
    pub fn descriptor(&self) -> &dyn PluginDescriptor {
        &*self.descriptor
    }

    #[inline]
    pub fn as_raw(&self) -> &clap_plugin_descriptor {
        &self.raw_descriptor
    }

    #[inline]
    pub fn id(&self) -> &CStr {
        self.descriptor.id()
    }
}

// SAFETY: the whole struct is immutable.
unsafe impl Send for PluginDescriptorWrapper {}
// SAFETY: the whole struct is immutable.
unsafe impl Sync for PluginDescriptorWrapper {}

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
    pub const FEATURE_LIMITER: &CStr = CLAP_PLUGIN_FEATURE_LIMITER;

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
