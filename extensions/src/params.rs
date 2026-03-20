//! This extension allows plugins to expose parameters to hosts,
//! which can then display them to the user and allow them to be automated and modulated.
//!
//! ## Main idea
//!
//! The host sees the plugin as an atomic entity; and acts as a controller on top of its parameters.
//! The plugin is responsible for keeping its audio processor and its GUI in sync.  
//!
//! The host can at any time read parameters' value using [`PluginParams::get_value`].
//!
//! There are two options to communicate parameter value changes, and they are not concurrent:
//! - sending automation points during [`PluginAudioProcessor::process`](clack_plugin::plugin::PluginAudioProcessor::process)
//! - sending automation points during [`PluginMainThreadParams::flush`] or [`PluginAudioProcessorParams::flush`], for parameter changes without processing audio
//!
//! When the plugin changes a parameter value, it must inform the host.
//! The plugin will send a [`ParamValueEvent`](clack_plugin::events::event_types::ParamValueEvent) event during `process()` or `flush()`.
//! If the parameter change was generated from a user interaction, don't forget to mark the beginning and end
//! of the gesture by sending [`ParamGestureBeginEvent`](clack_plugin::events::event_types::ParamGestureBeginEvent) and [`ParamGestureEndEvent`](clack_plugin::events::event_types::ParamGestureEndEvent) events.
//!
//! ## MIDI CC handling
//!
//! MIDI CCs are tricky because you may not know when the parameter adjustment ends.
//! Also if the host records incoming MIDI CC and parameter change automation at the same time,
//! there will be a conflict at playback: MIDI CC vs Automation.
//! The parameter automation will always target the same parameter because the param_id is stable.
//! The MIDI CC may have a different mapping in the future and may result in a different playback.
//!
//! When a MIDI CC changes a parameter's value, set the [`EventFlags::DONT_RECORD`](clack_plugin::events::EventFlags::DONT_RECORD) flag.
//! That way the host may record the MIDI CC automation, but not the parameter change and there won't be conflict at playback.
//!
//! ## Scenarios
//!
//! ### Loading a preset
//! - load the preset in a temporary state
//! - call [`HostParams::rescan`] if anything changed
//! - call [`HostLatency::changed`](crate::latency::HostLatency::changed) if latency changed
//! - invalidate any other info that may be cached by the host
//! - if the plugin is activated and the preset will introduce breaking changes
//!   (latency, audio ports, new parameters, ...) be sure to wait for the host
//!   to deactivate the plugin to apply those changes.
//!   If there are no breaking changes, the plugin can apply them them right away.
//!   The plugin is responsible for updating both its audio processor and its gui.
//!
//! ### Turning a knob on the DAW interface
//! - the host will send an automation event to the plugin via a call to either [`PluginAudioProcessor::process`](clack_plugin::plugin::PluginAudioProcessor::process) or during a flush (see above).
//!
//! ### Turning a knob on the Plugin interface
//! - the plugin is responsible for sending the parameter value to its audio processor
//! - call [`HostParams::request_flush`] or [`HostSharedHandle::request_process`](clack_plugin::host::HostSharedHandle::request_process).
//! - when the host calls either [`PluginAudioProcessor::process`](clack_plugin::plugin::PluginAudioProcessor::process) or a flush callback (see above),
//!   send an automation event and don't forget to wrap the parameter change(s)
//!   with [`ParamGestureBeginEvent`](clack_plugin::events::event_types::ParamGestureBeginEvent)
//!   and [`ParamGestureEndEvent`](clack_plugin::events::event_types::ParamGestureEndEvent) events
//!   to define the beginning and end of the gesture.
//!
//! ### Turning a knob via automation
//! - host sends an automation point during [`PluginAudioProcessor::process`](clack_plugin::plugin::PluginAudioProcessor::process) or a flush callback (see above).
//! - the plugin is responsible for updating its GUI
//!
//! ### Turning a knob via plugin's internal MIDI mapping
//! - the plugin sends a [`ParamValueEvent`](clack_plugin::events::event_types::ParamValueEvent) output event, set [`EventFlags::DONT_RECORD`](clack_plugin::events::EventFlags::DONT_RECORD) flag
//! - the plugin is responsible for updating its GUI
//!
//! ### Adding or removing parameters
//! - if the plugin is activated call [`HostSharedHandle::request_restart`](clack_plugin::host::HostSharedHandle::request_restart).
//! - once the plugin isn't active:
//!   - apply the new state
//!   - if a parameter is gone or is created with an id that may have been used before,
//!     call [`HostParams::clear`] with [`ParamClearFlags::ALL`]
//!   - call [`HostParams::rescan`] with [`ParamRescanFlags::ALL`]
//!
//! ## Persisting parameter values
//!
//! Plugins are responsible for persisting their parameter's values between
//! sessions by implementing the state extension. Otherwise parameter value will
//! not be recalled when reloading a project. Hosts should _not_ try to save and
//! restore parameter values for plugins that don't implement the state
//! extension.
//!
//! A host might use stable IDs provided by the plugin to save and restore extra data related to parameters (such as automation tracks, modulation info, etc.).
//! A plugin must _not_ change the stable ID of an existing parameter, otherwise the host won't be able to restore the related data correctly.
//!
//! ### Parameter range changes between releases
//!
//! CLAP allows the plugin to change the parameter range, yet the plugin developer
//! should be aware that doing so isn't without risk, especially if you made the
//! promise to never change the sound. If you want to be 100% certain that the
//! sound will not change with all host, then simply never change the range.
//!
//! There are two approaches to automations, either you automate the plain value,
//! or you automate the knob position. The first option will be robust to a range
//! increase, while the second won't be.
//!
//! If the host goes with the second approach (automating the knob position), it means
//! that the plugin is hosted in a relaxed environment regarding sound changes (they are
//! accepted, and not a concern as long as they are reasonable). Though, stepped parameters
//! should be stored as plain value in the document.
//!
//! If the host goes with the first approach, there will still be situation where the
//! sound may inevitably change. For example, if the plugin increase the range, there
//! is an automation playing at the max value and on top of that an LFO is applied.
//! See the following curve:
//! ```
//!                                   .
//!                                  . .
//!          .....                  .   .
//! before: .     .     and after: .     .
//! ```
//!
//!
//! ## Advice for the host
//!
//! - store plain values in the document (automation)
//! - store modulation amount in plain value delta, not in percentage
//! - when you apply a CC mapping, remember the min/max plain values so you can adjust
//! - do not implement a parameter saving fall back for plugins that don't
//!   implement the [state](crate::state) extension
//!
//! ## Advice for the plugin
//!
//! - think carefully about your parameter range when designing your DSP
//! - avoid shrinking parameter ranges, they are very likely to change the sound
//! - consider changing the parameter range as a tradeoff: what you improve vs what you break
//! - make sure to implement saving and loading the parameter values using the
//!   [state](crate::state) extension
//! - if you plan to use adapters for other plugin formats, then you need to pay extra
//!   attention to the adapter requirements

use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clack_common::utils::{ClapId, Cookie};
use clap_sys::ext::params::*;
use std::ffi::CStr;

bitflags! {
    /// Flags to indicate what parameter information has changed and needs to be rescanned by the host.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamRescanFlags: u32 {
        /// The parameter values have changed, e.g. after loading a preset.
        /// The host will scan all the parameter values.
        /// The host will not record those changes as automation points.
        const VALUES = CLAP_PARAM_RESCAN_VALUES;

        /// The parameter's info has changed (e.g. name, module, ranges).
        const INFO = CLAP_PARAM_RESCAN_INFO;

        /// The parameter's value to text conversion has changed.
        const TEXT = CLAP_PARAM_RESCAN_TEXT;

        /// Invalidates everything the host knows about parameters.
        /// This can only be used while the plugin is deactivated.
        const ALL = CLAP_PARAM_RESCAN_ALL;
    }
}

impl ParamRescanFlags {
    /// Returns `true` if any of the given flags that are set imply that a plugin instance's restart
    /// is needed before params can be rescanned.
    #[inline]
    pub const fn requires_restart(&self) -> bool {
        self.contains(Self::ALL)
    }
}

bitflags! {
    /// Flags to indicate what references to a parameter should be cleared by the host.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamClearFlags: u32 {
        /// Clears all possible references to a parameter, including automation and modulation.
        const ALL = CLAP_PARAM_CLEAR_ALL;
        /// Clears all automation for a parameter.
        const AUTOMATIONS = CLAP_PARAM_CLEAR_AUTOMATIONS;
        /// Clears all modulation for a parameter.
        const MODULATIONS = CLAP_PARAM_CLEAR_MODULATIONS;
    }
}

bitflags! {
    /// Flags providing additional information about a specific parameter.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ParamInfoFlags: u32 {
        /// Automation can be recorded for this parameter.
        const IS_AUTOMATABLE = CLAP_PARAM_IS_AUTOMATABLE;

        /// This parameter supports per-channel automation.
        const IS_AUTOMATABLE_PER_CHANNEL = CLAP_PARAM_IS_AUTOMATABLE_PER_CHANNEL;

        /// This parameter supports per-key automation.
        const IS_AUTOMATABLE_PER_KEY = CLAP_PARAM_IS_AUTOMATABLE_PER_KEY;

        /// This parameter supports per-note automation.
        const IS_AUTOMATABLE_PER_NOTE_ID = CLAP_PARAM_IS_AUTOMATABLE_PER_NOTE_ID;

        /// This parameter supports per-port automation.
        const IS_AUTOMATABLE_PER_PORT = CLAP_PARAM_IS_AUTOMATABLE_PER_PORT;

        /// This parameter is used to merge the plugin and host bypass button.
        /// It implies that the parameter is stepped, with `0.0` being bypass off, and `1.0` being bypass on.
        const IS_BYPASS = CLAP_PARAM_IS_BYPASS;

        /// This parameter should not be shown to the user, because it is currently not used.
        /// It is not necessary to process automation for this parameter.
        const IS_HIDDEN = CLAP_PARAM_IS_HIDDEN;

        /// This parameter supports modulation.
        const IS_MODULATABLE = CLAP_PARAM_IS_MODULATABLE;

        /// This parameter supports per-channel modulation.
        const IS_MODULATABLE_PER_CHANNEL = CLAP_PARAM_IS_MODULATABLE_PER_CHANNEL;

        /// This parameter supports per-key modulation.
        const IS_MODULATABLE_PER_KEY = CLAP_PARAM_IS_MODULATABLE_PER_KEY;

        /// This parameter supports per-note modulation.
        const IS_MODULATABLE_PER_NOTE_ID = CLAP_PARAM_IS_MODULATABLE_PER_NOTE_ID;

        /// This parameter supports per-port modulation.
        const IS_MODULATABLE_PER_PORT = CLAP_PARAM_IS_MODULATABLE_PER_PORT;

        /// This parameter is periodic, e.g. a phase.
        const IS_PERIODIC = CLAP_PARAM_IS_PERIODIC;

        /// This parameter cannot be changed by the host.
        const IS_READONLY = CLAP_PARAM_IS_READONLY;

        /// This parameter is stepped (integer values only).
        /// If so, the double value is converted to integer using a cast (equivalent to `trunc`).
        const IS_STEPPED = CLAP_PARAM_IS_STEPPED;

        /// Any change to this parameter will affect the plugin's output and requires it to be
        /// processed via `process()` if the plugin is active.
        const REQUIRES_PROCESS = CLAP_PARAM_REQUIRES_PROCESS;

        /// This parameter represents an enumeration of discrete values.
        const IS_ENUM = CLAP_PARAM_IS_ENUM;
    }
}

impl ParamInfoFlags {
    /// Flags that, when changed, require the plugin to call the host rescan function with [`ParamRescanFlags::INFO`].
    pub const FLAGS_REQUIRING_INFO_RESCAN: Self =
        Self::from_bits_truncate(Self::IS_PERIODIC.bits() | Self::IS_HIDDEN.bits());

    /// Flags that, when changed, require the plugin to call the host rescan function with [`ParamRescanFlags::ALL`].
    pub const FLAGS_REQUIRING_FULL_RESCAN: Self = Self::from_bits_truncate(
        Self::IS_AUTOMATABLE.bits()
            | Self::IS_AUTOMATABLE_PER_NOTE_ID.bits()
            | Self::IS_AUTOMATABLE_PER_KEY.bits()
            | Self::IS_AUTOMATABLE_PER_CHANNEL.bits()
            | Self::IS_AUTOMATABLE_PER_PORT.bits()
            | Self::IS_MODULATABLE.bits()
            | Self::IS_MODULATABLE_PER_NOTE_ID.bits()
            | Self::IS_MODULATABLE_PER_KEY.bits()
            | Self::IS_MODULATABLE_PER_CHANNEL.bits()
            | Self::IS_MODULATABLE_PER_PORT.bits()
            | Self::IS_READONLY.bits()
            | Self::IS_BYPASS.bits()
            | Self::IS_STEPPED.bits(),
    );
}

/// The Plugin-side of the Params extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginParams(RawExtension<PluginExtensionSide, clap_plugin_params>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginParams {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_PARAMS];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// The Host-side of the Params extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostParams(RawExtension<HostExtensionSide, clap_host_params>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostParams {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_PARAMS];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}
/// Information about a parameter.
pub struct ParamInfo<'a> {
    /// A stable identifier for the parameter, which must never change.
    pub id: ClapId,
    /// Flags providing more information about the parameter.
    pub flags: ParamInfoFlags,
    /// An opaque pointer that can be used by the plugin to quickly access the parameter's data.
    ///
    /// This value is optional and set by the plugin. Its purpose is to provide fast access to the
    /// plugin parameter object by caching its pointer.
    ///
    /// The cookie is invalidated by a call to `clap_host_params.rescan(CLAP_PARAM_RESCAN_ALL)` or
    /// when the plugin is destroyed.
    pub cookie: Cookie,
    /// The display name of the parameter, e.g. "Volume".
    pub name: &'a [u8],
    /// The module path of the parameter, e.g. "Oscillators/Wavetable 1".
    /// The host can use `/` as a separator to show a tree-like structure.
    pub module: &'a [u8],
    /// The minimum plain value of the parameter.
    pub min_value: f64,
    /// The maximum plain value of the parameter.
    pub max_value: f64,
    /// The default plain value of the parameter.
    pub default_value: f64,
}

impl<'a> ParamInfo<'a> {
    /// Gets a [`ParamInfo`] from a reference to a raw, C-FFI compatible parameter info buffer.
    pub fn from_raw(raw: &'a clap_param_info) -> Option<Self> {
        Some(Self {
            id: ClapId::from_raw(raw.id)?,
            flags: ParamInfoFlags::from_bits_truncate(raw.flags),
            cookie: Cookie::from_raw(raw.cookie),
            name: crate::utils::data_from_array_buf(&raw.name),
            module: crate::utils::data_from_array_buf(&raw.module),
            min_value: raw.min_value,
            max_value: raw.max_value,
            default_value: raw.default_value,
        })
    }

    /// Computes the difference between this and another [`ParamInfo`], and returns a set of
    /// [`ParamRescanFlags`] describing which parameter information need to be rescanned.
    pub fn diff_for_rescan(&self, other: &ParamInfo) -> ParamRescanFlags {
        #[inline]
        fn flags_differ(
            a: ParamInfoFlags,
            b: ParamInfoFlags,
            flags_to_check: ParamInfoFlags,
        ) -> bool {
            a.intersection(flags_to_check) != b.intersection(flags_to_check)
        }

        let mut flags = ParamRescanFlags::empty();

        if self.name != other.name
            || self.module != other.module
            || flags_differ(
                self.flags,
                other.flags,
                ParamInfoFlags::FLAGS_REQUIRING_INFO_RESCAN,
            )
        {
            flags |= ParamRescanFlags::INFO;
        }

        if self.min_value != other.min_value
            || self.max_value != other.max_value
            || self.cookie != other.cookie
            || flags_differ(
                self.flags,
                other.flags,
                ParamInfoFlags::FLAGS_REQUIRING_FULL_RESCAN,
            )
        {
            flags |= ParamRescanFlags::ALL
        }

        flags
    }
}

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
