//! This extension can be used to specify the surround channel mapping used by the plugin.

#![deny(missing_docs)]

use crate::audio_ports::AudioPortType;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::surround::*;
use std::ffi::CStr;

/// The Plugin-side of the Surround extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginSurround(RawExtension<PluginExtensionSide, clap_plugin_surround>);

/// The Host-side of the Surround extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostSurround(RawExtension<HostExtensionSide, clap_host_surround>);

// SAFETY: The type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginSurround {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_SURROUND, CLAP_EXT_SURROUND_COMPAT];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: This type is expected to contain a type that is ABI-compatible with the matching extension type.
        Self(unsafe { raw.cast() })
    }
}

// SAFETY: The type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostSurround {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_SURROUND, CLAP_EXT_SURROUND_COMPAT];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: This type is expected to contain a type that is ABI-compatible with the matching extension type.
        Self(unsafe { raw.cast() })
    }
}

/// A specific surround channel.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::cast_possible_truncation)] // The CLAP spec defines these as u32, but there are only 20 valid values, so they will always fit in an u8.
pub enum SurroundChannel {
    /// Front left speaker.
    FrontLeft = CLAP_SURROUND_FL as u8,
    /// Front right speaker.
    FrontRight = CLAP_SURROUND_FR as u8,
    /// Front center speaker.
    FrontCenter = CLAP_SURROUND_FC as u8,
    /// Low frequency speaker (subwoofer).
    LowFrequency = CLAP_SURROUND_LFE as u8,
    /// Back left speaker.
    BackLeft = CLAP_SURROUND_BL as u8,
    /// Back right speaker.
    BackRight = CLAP_SURROUND_BR as u8,
    /// Front center-left speaker.
    FrontLeftCenter = CLAP_SURROUND_FLC as u8,
    /// Front center-right speaker.
    FrontRightCenter = CLAP_SURROUND_FRC as u8,
    /// Back center speaker.
    BackCenter = CLAP_SURROUND_BC as u8,
    /// Side left speaker.
    SideLeft = CLAP_SURROUND_SL as u8,
    /// Side right speaker.
    SideRight = CLAP_SURROUND_SR as u8,
    /// Top center speaker.
    TopCenter = CLAP_SURROUND_TC as u8,
    /// Top front left speaker.
    TopFrontLeft = CLAP_SURROUND_TFL as u8,
    /// Top front center speaker.
    TopFrontCenter = CLAP_SURROUND_TFC as u8,
    /// Top front right speaker.
    TopFrontRight = CLAP_SURROUND_TFR as u8,
    /// Top back left speaker.
    TopBackLeft = CLAP_SURROUND_TBL as u8,
    /// Top back center speaker.
    TopBackCenter = CLAP_SURROUND_TBC as u8,
    /// Top back right speaker.
    TopBackRight = CLAP_SURROUND_TBR as u8,
    /// Top side left speaker.
    TopSideLeft = 18, // CLAP_SURROUND_TSL as u8,
    /// Top side right speaker.
    TopSideRight = 19, //CLAP_SURROUND_TSR as u8,
}

bitflags::bitflags! {
    /// A mask containing multiple surround channels.
    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct SurroundChannels: u64 {
        /// See [`SurroundChannel::FrontLeft`].
        const FRONT_LEFT = 1u64 << CLAP_SURROUND_FL;
        /// See [`SurroundChannel::FrontRight`].
        const FRONT_RIGHT = 1u64 << CLAP_SURROUND_FR;
        /// See [`SurroundChannel::FrontCenter`].
        const FRONT_CENTER = 1u64 << CLAP_SURROUND_FC;
        /// See [`SurroundChannel::LowFrequency`].
        const LOW_FREQUENCY = 1u64 << CLAP_SURROUND_LFE;
        /// See [`SurroundChannel::BackLeft`].
        const BACK_LEFT = 1u64 << CLAP_SURROUND_BL;
        /// See [`SurroundChannel::BackRight`].
        const BACK_RIGHT = 1u64 << CLAP_SURROUND_BR;
        /// See [`SurroundChannel::FrontLeftCenter`].
        const FRONT_LEFT_CENTER = 1u64 << CLAP_SURROUND_FLC;
        /// See [`SurroundChannel::FrontRightCenter`].
        const FRONT_RIGHT_CENTER = 1u64 << CLAP_SURROUND_FRC;
        /// See [`SurroundChannel::BackCenter`].
        const BACK_CENTER = 1u64 << CLAP_SURROUND_BC;
        /// See [`SurroundChannel::SideLeft`].
        const SIDE_LEFT = 1u64 << CLAP_SURROUND_SL;
        /// See [`SurroundChannel::SideRight`].
        const SIDE_RIGHT = 1u64 << CLAP_SURROUND_SR;
        /// See [`SurroundChannel::TopCenter`].
        const TOP_CENTER = 1u64 << CLAP_SURROUND_TC;
        /// See [`SurroundChannel::TopFrontLeft`].
        const TOP_FRONT_LEFT = 1u64 << CLAP_SURROUND_TFL;
        /// See [`SurroundChannel::TopFrontCenter`].
        const TOP_FRONT_CENTER = 1u64 << CLAP_SURROUND_TFC;
        /// See [`SurroundChannel::TopFrontRight`].
        const TOP_FRONT_RIGHT = 1u64 << CLAP_SURROUND_TFR;
        /// See [`SurroundChannel::TopBackLeft`].
        const TOP_BACK_LEFT = 1u64 << CLAP_SURROUND_TBL;
        /// See [`SurroundChannel::TopBackCenter`].
        const TOP_BACK_CENTER = 1u64 << CLAP_SURROUND_TBC;
        /// See [`SurroundChannel::TopBackRight`].
        const TOP_BACK_RIGHT = 1u64 << CLAP_SURROUND_TBR;
        /// See [`SurroundChannel::TopSideLeft`].
        const TOP_SIDE_LEFT = 1u64 << 18;  // CLAP_SURROUND_TSL, clap_sys is somewhat outdated;
        /// See [`SurroundChannel::TopSideRight`].
        const TOP_SIDE_RIGHT = 1u64 << 19; // CLAP_SURROUND_TSR;
    }
}

impl SurroundChannel {
    /// Convert a raw u8 value to a [`SurroundChannel`], if it corresponds to a valid channel.
    #[inline]
    pub fn from_raw(raw: u8) -> Option<Self> {
        match raw as u32 {
            CLAP_SURROUND_FL => Some(SurroundChannel::FrontLeft),
            CLAP_SURROUND_FR => Some(SurroundChannel::FrontRight),
            CLAP_SURROUND_FC => Some(SurroundChannel::FrontCenter),
            CLAP_SURROUND_LFE => Some(SurroundChannel::LowFrequency),
            CLAP_SURROUND_BL => Some(SurroundChannel::BackLeft),
            CLAP_SURROUND_BR => Some(SurroundChannel::BackRight),
            CLAP_SURROUND_FLC => Some(SurroundChannel::FrontLeftCenter),
            CLAP_SURROUND_FRC => Some(SurroundChannel::FrontRightCenter),
            CLAP_SURROUND_BC => Some(SurroundChannel::BackCenter),
            CLAP_SURROUND_SL => Some(SurroundChannel::SideLeft),
            CLAP_SURROUND_SR => Some(SurroundChannel::SideRight),
            CLAP_SURROUND_TC => Some(SurroundChannel::TopCenter),
            CLAP_SURROUND_TFL => Some(SurroundChannel::TopFrontLeft),
            CLAP_SURROUND_TFC => Some(SurroundChannel::TopFrontCenter),
            CLAP_SURROUND_TFR => Some(SurroundChannel::TopFrontRight),
            CLAP_SURROUND_TBL => Some(SurroundChannel::TopBackLeft),
            CLAP_SURROUND_TBC => Some(SurroundChannel::TopBackCenter),
            CLAP_SURROUND_TBR => Some(SurroundChannel::TopBackRight),
            18 => Some(SurroundChannel::TopSideLeft),
            19 => Some(SurroundChannel::TopSideRight),
            _ => None,
        }
    }

    /// Convert this [`SurroundChannel`] to its raw u8 representation.
    #[inline]
    pub fn to_raw(self) -> u8 {
        self as _
    }

    /// Convert a raw byte slice to a slice of [`SurroundChannel`]s, if every value in the slice corresponds to a valid channel.
    #[inline]
    pub fn slice_from_raw(slice: &[u8]) -> Option<&[SurroundChannel]> {
        for value in slice {
            SurroundChannel::from_raw(*value)?;
        }

        // SAFETY: Every value in the slice has been verified to be a valid SurroundChannel, and SurroundChannel is repr(u8) and thus has the same memory layout as u8.
        unsafe {
            Some(std::slice::from_raw_parts(
                slice.as_ptr() as *const SurroundChannel,
                slice.len(),
            ))
        }
    }

    /// Convert this slice of [`SurroundChannel`]s to a slice of raw u8 values.
    #[inline]
    pub fn slice_as_raw(slice: &[SurroundChannel]) -> &[u8] {
        // SAFETY: SurroundChannel is repr(u8) and thus has the same memory layout as u8.
        unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len()) }
    }
}

impl From<SurroundChannel> for SurroundChannels {
    fn from(channel: SurroundChannel) -> Self {
        match channel {
            SurroundChannel::FrontLeft => SurroundChannels::FRONT_LEFT,
            SurroundChannel::FrontRight => SurroundChannels::FRONT_RIGHT,
            SurroundChannel::FrontCenter => SurroundChannels::FRONT_CENTER,
            SurroundChannel::LowFrequency => SurroundChannels::LOW_FREQUENCY,
            SurroundChannel::BackLeft => SurroundChannels::BACK_LEFT,
            SurroundChannel::BackRight => SurroundChannels::BACK_RIGHT,
            SurroundChannel::FrontLeftCenter => SurroundChannels::FRONT_LEFT_CENTER,
            SurroundChannel::FrontRightCenter => SurroundChannels::FRONT_RIGHT_CENTER,
            SurroundChannel::BackCenter => SurroundChannels::BACK_CENTER,
            SurroundChannel::SideLeft => SurroundChannels::SIDE_LEFT,
            SurroundChannel::SideRight => SurroundChannels::SIDE_RIGHT,
            SurroundChannel::TopCenter => SurroundChannels::TOP_CENTER,
            SurroundChannel::TopFrontLeft => SurroundChannels::TOP_FRONT_LEFT,
            SurroundChannel::TopFrontCenter => SurroundChannels::TOP_FRONT_CENTER,
            SurroundChannel::TopFrontRight => SurroundChannels::TOP_FRONT_RIGHT,
            SurroundChannel::TopBackLeft => SurroundChannels::TOP_BACK_LEFT,
            SurroundChannel::TopBackCenter => SurroundChannels::TOP_BACK_CENTER,
            SurroundChannel::TopBackRight => SurroundChannels::TOP_BACK_RIGHT,
            SurroundChannel::TopSideLeft => SurroundChannels::TOP_SIDE_LEFT,
            SurroundChannel::TopSideRight => SurroundChannels::TOP_SIDE_RIGHT,
        }
    }
}

impl Extend<SurroundChannel> for SurroundChannels {
    fn extend<T: IntoIterator<Item = SurroundChannel>>(&mut self, iter: T) {
        for channel in iter {
            *self |= SurroundChannels::from(channel);
        }
    }
}

impl FromIterator<SurroundChannel> for SurroundChannels {
    fn from_iter<I: IntoIterator<Item = SurroundChannel>>(iter: I) -> Self {
        let mut mask = SurroundChannels::empty();
        for channel in iter {
            mask |= SurroundChannels::from(channel);
        }
        mask
    }
}

impl AudioPortType<'static> {
    /// Surround audio port type.
    pub const SURROUND: Self = AudioPortType(CLAP_PORT_SURROUND);
}

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "configurable-audio-ports")]
mod config;
#[cfg(feature = "configurable-audio-ports")]
pub use config::*;
