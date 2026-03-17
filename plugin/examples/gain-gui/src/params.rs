//! Contains all types and implementations related to parameter management.

use crate::{GainPluginAudioProcessor, GainPluginMainThread};
use clack_extensions::params::*;
use clack_extensions::state::PluginStateImpl;
use clack_plugin::events::event_types::ParamValueEvent;
use clack_plugin::events::spaces::CoreEventSpace;
use clack_plugin::prelude::*;
use clack_plugin::stream::{InputStream, OutputStream};
use clack_plugin::utils::Cookie;
use std::ffi::CStr;
use std::fmt::Write as _;
use std::io::{Read, Write as _};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// The default value of the volume parameter.
const DEFAULT_VOLUME: f32 = 1.0;

/// A struct that manages the parameters for our plugin.
///
/// For now, it only manages a single, `volume` parameter.
///
/// This struct will be used both by the [`GainPluginMainThread`] (which the host will use
/// to query the value of our parameters), and by the [`GainPluginAudioProcessor`], which will
/// actually modulate the audio samples.
pub struct GainParamsShared {
    /// The current value of the volume parameter.
    volume: AtomicF32,
    /// Whether a gesture is currently active or not on the `volume` parameter
    has_gesture: AtomicBool,
}

impl GainParamsShared {
    /// The unique identifier for the Volume parameter.
    pub const PARAM_VOLUME_ID: ClapId = ClapId::new(1);

    /// Initializes the shared parameter value.
    pub fn new() -> Self {
        Self {
            volume: AtomicF32::new(DEFAULT_VOLUME),
            has_gesture: AtomicBool::new(false),
        }
    }
}

/// A param gesture change type
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GestureChange {
    /// User started changing a parameter
    Begin,
    /// User finished changing a parameter
    End,
    /// User both started and finished changing a parameter during the current block
    Both,
}

/// The local-side of parameter state.
///
/// This state is local to the current thread (whether it is the main-thread, the UI or the audio
/// thread), it is not shared directly with the others.
///
/// This allows us to both check for differences, and to only update parameter state when we want
/// to.
pub struct GainParamsLocal {
    /// The local value of the volume parameter.
    volume: f32,
    /// Whether the user is currently changing the parameter, that we know of
    pub has_gesture: bool,
}

impl GainParamsLocal {
    /// Initializes a new local state from the current shared state.
    pub fn new(shared: &GainParamsShared) -> Self {
        Self {
            volume: shared.volume.load(),
            has_gesture: shared.has_gesture.load(Ordering::Relaxed),
        }
    }

    /// Returns the current local volume value.
    #[inline]
    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    /// Sets the current local volume value to `new_volume`.
    ///
    /// It is clamped to the range `0..=1.0`.
    #[inline]
    pub fn set_volume(&mut self, new_volume: f32) {
        self.volume = new_volume.clamp(0.0, 1.0);
    }

    /// Fetch updates from the `shared` state.
    ///
    /// If the volume parameter has been updated, this returns `true`.
    #[inline]
    pub fn fetch_updates(&mut self, shared: &GainParamsShared) -> bool {
        let latest_volume = shared.volume.load();

        if latest_volume != self.volume {
            self.volume = latest_volume;
            true
        } else {
            false
        }
    }

    /// Pushes the local volume parameter value to the `shared` state.
    ///
    /// If the values were different and an actual update occurred, this returns `true`.
    #[inline]
    pub fn push_updates(&self, shared: &GainParamsShared) -> bool {
        let previous_value = shared.volume.swap(self.volume);

        previous_value != self.volume
    }

    /// Pushes the local gesture state to the `shared` state.
    #[inline]
    pub fn push_gesture(&self, shared: &GainParamsShared) {
        shared
            .has_gesture
            .store(self.has_gesture, Ordering::Relaxed);
    }

    /// Fetches updates to the gesture state.
    ///
    /// If a gesture state changed occurred, it is returned.
    /// If the gesture did not change from the last update, `None` is returned.
    #[inline]
    pub fn fetch_gesture(
        &mut self,
        shared: &GainParamsShared,
        has_ui_param_updates: bool,
    ) -> Option<GestureChange> {
        let previous_gesture = self.has_gesture;

        self.has_gesture = shared.has_gesture.load(Ordering::Relaxed);

        if previous_gesture == self.has_gesture {
            return if has_ui_param_updates && !self.has_gesture {
                Some(GestureChange::Both)
            } else {
                None
            };
        }

        if previous_gesture {
            Some(GestureChange::End)
        } else {
            Some(GestureChange::Begin)
        }
    }

    /// Handles incoming events.
    ///
    /// If the given event is a matching parameter change event, the volume parameter will be
    /// updated accordingly.
    pub fn handle_event(&mut self, event: &UnknownEvent) {
        if let Some(CoreEventSpace::ParamValue(event)) = event.as_core_event() {
            if event.param_id() == GainParamsShared::PARAM_VOLUME_ID {
                self.set_volume(event.value() as f32);
            }
        }
    }

    /// Sends the value of the volume parameter to the host, via a [`ParamValueEvent`].
    pub fn send_param_events(&self, output_events: &mut OutputEvents) {
        let event = ParamValueEvent::new(
            0,
            GainParamsShared::PARAM_VOLUME_ID,
            Pckn::match_all(),
            self.volume as f64,
            Cookie::empty(),
        );

        let _ = output_events.try_push(event);
    }
}

/// Implementation of the State extension.
///
/// Our state "serialization" is extremely simple and basic: we only have the value of the
/// volume parameter to store, so we just store its bytes (in little-endian) and call it a day.
impl PluginStateImpl for GainPluginMainThread<'_> {
    fn save(&mut self, output: &mut OutputStream) -> Result<(), PluginError> {
        self.params.fetch_updates(&self.shared.params);
        let volume_param = self.params.get_volume();

        output.write_all(&volume_param.to_le_bytes())?;
        Ok(())
    }

    fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError> {
        let mut buf = [0; 4];
        input.read_exact(&mut buf)?;
        let volume_value = f32::from_le_bytes(buf);
        self.params.set_volume(volume_value);
        self.params.push_updates(&self.shared.params);
        Ok(())
    }
}

impl PluginMainThreadParams for GainPluginMainThread<'_> {
    fn count(&mut self) -> u32 {
        1
    }

    fn get_info(&mut self, param_index: u32, info: &mut ParamInfoWriter) {
        if param_index != 0 {
            return;
        }
        info.set(&ParamInfo {
            id: 1.into(),
            flags: ParamInfoFlags::IS_AUTOMATABLE,
            cookie: Default::default(),
            name: b"Volume",
            module: b"",
            min_value: 0.0,
            max_value: 1.0,
            default_value: DEFAULT_VOLUME as f64,
        })
    }

    fn get_value(&mut self, param_id: ClapId) -> Option<f64> {
        if param_id == 1 {
            Some(self.params.get_volume() as f64)
        } else {
            None
        }
    }

    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> std::fmt::Result {
        if param_id == 1 {
            write!(writer, "{0:.2} %", value * 100.0)
        } else {
            Err(std::fmt::Error)
        }
    }

    fn text_to_value(&mut self, param_id: ClapId, text: &CStr) -> Option<f64> {
        let text = text.to_str().ok()?;
        if param_id == 1 {
            let text = text.strip_suffix('%').unwrap_or(text).trim();
            let percentage: f64 = text.parse().ok()?;

            Some(percentage / 100.0)
        } else {
            None
        }
    }

    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        _output_parameter_changes: &mut OutputEvents,
    ) {
        for event in input_parameter_changes {
            self.params.handle_event(event)
        }
    }
}

/// A small helper to atomically load and store an `f32` value.
struct AtomicF32(AtomicU32);

impl AtomicF32 {
    /// Creates a new atomic `f32`.
    #[inline]
    fn new(value: f32) -> Self {
        Self(AtomicU32::new(f32_to_u32_bytes(value)))
    }

    /// Stores the given `value`, and returns the previously stored one.
    #[inline]
    fn swap(&self, value: f32) -> f32 {
        f32_from_u32_bytes(self.0.swap(f32_to_u32_bytes(value), Ordering::Relaxed))
    }

    /// Loads the contained `value`.
    #[inline]
    fn load(&self) -> f32 {
        f32_from_u32_bytes(self.0.load(Ordering::Relaxed))
    }
}

/// Packs a `f32` into the bytes of an `u32`.
///
/// The resulting value is meaningless and should not be used directly,
/// except for unpacking with [`f32_from_u32_bytes`].
///
/// This is an internal helper used by [`AtomicF32`].
#[inline]
fn f32_to_u32_bytes(value: f32) -> u32 {
    u32::from_ne_bytes(value.to_ne_bytes())
}

/// The counterpart to [`f32_to_u32_bytes`].
#[inline]
fn f32_from_u32_bytes(bytes: u32) -> f32 {
    f32::from_ne_bytes(bytes.to_ne_bytes())
}
