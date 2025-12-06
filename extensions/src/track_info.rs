//#![deny(warnings)]

use crate::audio_ports::AudioPortType;
use bitflags::bitflags;
use clack_common::extensions::*;
use clack_common::utils::Color;
use clap_sys::ext::track_info::*;
use std::ffi::CStr;

/// Plugin-side of the Track Info extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginTrackInfo(RawExtension<PluginExtensionSide, clap_plugin_track_info>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginTrackInfo {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_TRACK_INFO, CLAP_EXT_TRACK_INFO_COMPAT];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// Host-side of the Track Info extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostTrackInfo(RawExtension<HostExtensionSide, clap_host_track_info>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostTrackInfo {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_TRACK_INFO, CLAP_EXT_TRACK_INFO_COMPAT];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

bitflags! {
    /// Option flags for [`TrackInfo`].
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TrackInfoFlags: u64 {
        const HAS_TRACK_NAME = CLAP_TRACK_INFO_HAS_TRACK_NAME;
        const HAS_TRACK_COLOR = CLAP_TRACK_INFO_HAS_TRACK_COLOR;
        const HAS_AUDIO_CHANNEL = CLAP_TRACK_INFO_HAS_AUDIO_CHANNEL;

        const IS_FOR_RETURN_TRACK = CLAP_TRACK_INFO_IS_FOR_RETURN_TRACK;
        const IS_FOR_BUS = CLAP_TRACK_INFO_IS_FOR_BUS;
        const IS_FOR_MASTER = CLAP_TRACK_INFO_IS_FOR_MASTER;
    }
}

pub struct TrackInfo<'a, 'n> {
    flags: TrackInfoFlags,
    name: Option<&'n [u8]>,
    color: Color,
    audio_channel_count: u32,
    audio_port_type: Option<AudioPortType<'a>>,
}

impl<'a, 'n> TrackInfo<'a, 'n> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            flags: TrackInfoFlags::empty(),
            name: None,
            color: Color::TRANSPARENT,
            audio_channel_count: 0,
            audio_port_type: None,
        }
    }

    #[inline]
    pub unsafe fn from_raw(raw: &'n clap_track_info) -> Self {
        let flags = TrackInfoFlags::from_bits_truncate(raw.flags);

        let (port_type, channel_count) = if flags.contains(TrackInfoFlags::HAS_AUDIO_CHANNEL) {
            (
                AudioPortType::from_raw(raw.audio_port_type),
                raw.audio_channel_count,
            )
        } else {
            (None, 0)
        };

        let color = if flags.contains(TrackInfoFlags::HAS_TRACK_COLOR) {
            Color::from_raw(&raw.color)
        } else {
            Color::TRANSPARENT
        };

        Self {
            flags,
            name: flags
                .contains(TrackInfoFlags::HAS_TRACK_NAME)
                .then(|| data_from_array_buf(&raw.name)),
            color,
            audio_port_type: port_type,
            audio_channel_count: channel_count as u32, // TODO
        }
    }

    #[inline]
    pub const fn flags(&self) -> TrackInfoFlags {
        self.flags
    }

    #[inline]
    pub const fn set_flags(&mut self, flags: TrackInfoFlags) {
        self.flags = flags;
    }

    #[inline]
    pub const fn with_flags(mut self, flags: TrackInfoFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub const fn name(&self) -> Option<&'n [u8]> {
        if !self.flags.contains(TrackInfoFlags::HAS_TRACK_NAME) {
            return None;
        }

        self.name
    }

    #[inline]
    pub const fn set_name(&mut self, name: Option<&'n [u8]>) {
        self.set_flag(TrackInfoFlags::HAS_TRACK_NAME, name.is_some());
        self.name = name;
    }

    #[inline]
    pub const fn with_name(mut self, name: Option<&'n [u8]>) -> Self {
        self.set_name(name);
        self
    }

    #[inline]
    pub const fn color(&self) -> Option<Color> {
        if !self.flags.contains(TrackInfoFlags::HAS_TRACK_COLOR) {
            return None;
        }

        Some(self.color)
    }

    #[inline]
    pub const fn set_color(&mut self, color: Option<Color>) {
        self.set_flag(TrackInfoFlags::HAS_TRACK_COLOR, color.is_some());

        self.color = match color {
            Some(color) => color,
            None => Color::TRANSPARENT,
        };
    }

    #[inline]
    pub const fn with_color(mut self, color: Option<Color>) -> Self {
        self.set_color(color);
        self
    }

    #[inline]
    pub const fn audio_channel_count(&self) -> Option<u32> {
        if !self.flags.contains(TrackInfoFlags::HAS_AUDIO_CHANNEL) {
            return None;
        }

        Some(self.audio_channel_count)
    }

    #[inline]
    pub const fn audio_port_type(&self) -> Option<AudioPortType<'a>> {
        if !self.flags.contains(TrackInfoFlags::HAS_AUDIO_CHANNEL) {
            return None;
        }

        self.audio_port_type
    }

    #[inline]
    pub const fn set_audio_channels(
        &mut self,
        audio_port_type: Option<AudioPortType<'a>>,
        channel_count: u32,
    ) {
        self.set_flag(TrackInfoFlags::HAS_AUDIO_CHANNEL, audio_port_type.is_some());

        self.audio_channel_count = if audio_port_type.is_some() {
            channel_count
        } else {
            0
        };

        self.audio_port_type = audio_port_type;
    }

    #[inline]
    pub const fn with_audio_channels(
        mut self,
        audio_port_type: Option<AudioPortType<'a>>,
        channel_count: u32,
    ) -> Self {
        self.set_audio_channels(audio_port_type, channel_count);
        self
    }

    #[inline]
    const fn set_flag(&mut self, flag: TrackInfoFlags, value: bool) {
        self.flags = if value {
            self.flags.union(flag)
        } else {
            self.flags.difference(flag)
        };
    }
}

impl Default for TrackInfo<'_, '_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use crate::utils::write_to_array_buf;
    use clack_host::extensions::prelude::*;
    use std::marker::PhantomData;

    impl PluginTrackInfo {
        /// Notifies the plugin that its current track's info has changed.
        pub fn changed(&self, plugin: &mut PluginMainThreadHandle) {
            if let Some(changed) = plugin.use_extension(&self.0).changed {
                // SAFETY: This type guarantees the function pointer is valid, and
                // PluginMainThreadHandle guarantees the plugin pointer is valid
                unsafe { changed(plugin.as_raw()) }
            }
        }
    }

    pub struct TrackInfoWriter<'buf, 'port_type> {
        buffer: *mut clap_track_info,
        _buffer: PhantomData<&'buf mut clap_track_info>,
        _audio_port_type: PhantomData<AudioPortType<'port_type>>,
        is_set: bool,
    }

    impl<'port_type> TrackInfoWriter<'_, 'port_type> {
        #[inline]
        unsafe fn from_raw(ptr: *mut clap_track_info) -> Self {
            Self {
                buffer: ptr,
                _buffer: PhantomData,
                _audio_port_type: PhantomData,
                is_set: false,
            }
        }

        pub fn set(&mut self, track_info: &TrackInfo<'_, 'port_type>) {
            use core::ptr::write;

            let buf = self.buffer;

            // SAFETY: This type ensures the buf pointer is valid for writes and well-aligned.
            unsafe {
                write(&raw mut (*buf).flags, track_info.flags.bits());
                write(
                    &raw mut (*buf).audio_port_type,
                    track_info
                        .audio_port_type
                        .map(|p| p.as_raw())
                        .unwrap_or(core::ptr::null()),
                );
                write(
                    &raw mut (*buf).color,
                    track_info.color().unwrap_or(Color::TRANSPARENT).to_raw(),
                );
                write(
                    &raw mut (*buf).audio_channel_count,
                    track_info.audio_channel_count().unwrap_or(0) as i32, // TODO: i32 cast
                );
                write_to_array_buf(&raw mut (*buf).name, track_info.name().unwrap_or(b""))
            }

            self.is_set = true;
        }
    }

    /// Implementation of the Host-side of the Track Info extension.
    pub trait HostTrackInfoImpl {
        /// Gets info about the track the plugin belongs to.
        fn get<'a>(&'a mut self, writer: &mut TrackInfoWriter<'_, 'a>);
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<H> ExtensionImplementation<H> for HostTrackInfo
    where
        H: for<'a> HostHandlers<MainThread<'a>: HostTrackInfoImpl>,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_track_info {
                get: Some(get::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get<H>(host: *const clap_host, buf: *mut clap_track_info) -> bool
    where
        H: for<'a> HostHandlers<MainThread<'a>: HostTrackInfoImpl>,
    {
        HostWrapper::<H>::handle(host, |host| {
            let mut writer = TrackInfoWriter::from_raw(buf);
            host.main_thread().as_mut().get(&mut writer);
            Ok(writer.is_set)
        })
        .unwrap_or(false)
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::extensions::prelude::*;
    use std::mem::MaybeUninit;

    #[derive(Clone)]
    pub struct TrackInfoBuffer {
        inner: MaybeUninit<clap_track_info>,
    }

    impl Default for TrackInfoBuffer {
        #[inline]
        fn default() -> Self {
            Self::new()
        }
    }

    impl TrackInfoBuffer {
        #[inline]
        pub const fn new() -> Self {
            Self {
                inner: MaybeUninit::zeroed(),
            }
        }
    }

    impl HostTrackInfo {
        /// Indicates the plugin has changed its voice configuration, and the host needs to update
        /// it by calling [`get`](PluginVoiceInfoImpl::get) again.
        pub fn get<'host, 'buffer>(
            &self,
            host: &'host mut HostMainThreadHandle,
            buf: &'buffer mut TrackInfoBuffer,
        ) -> Option<TrackInfo<'host, 'buffer>> {
            let get = host.use_extension(&self.0).get?;

            // SAFETY: This type ensures the function pointer is valid.
            let success = unsafe { get(host.as_raw(), buf.inner.as_mut_ptr()) };
            if !success {
                return None;
            }

            // SAFETY: Per the CLAP spec, success being set guarantees that the buffer is initialized.
            // Worst case, the host didn't actually initialize some (or all) of these, and everything is
            // zeroed, which is a valid bit pattern for all fields of the struct.
            let raw = unsafe { buf.inner.assume_init_ref() };
            // SAFETY: Per the CLAP spec, a non-null audio_port_type is valid for read until the next call at least.
            unsafe { Some(TrackInfo::from_raw(raw)) }
        }
    }

    /// Implementation of the Plugin-side of the Track Info extension.
    pub trait PluginTrackInfoImpl {
        /// Informs the plugin that the Track Info has changed.
        fn changed(&self);
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P: Plugin> ExtensionImplementation<P> for PluginTrackInfo
    where
        for<'a> P: Plugin<MainThread<'a>: PluginTrackInfoImpl>,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_track_info {
                changed: Some(changed::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn changed<P>(plugin: *const clap_plugin)
    where
        for<'a> P: Plugin<MainThread<'a>: PluginTrackInfoImpl>,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            let plugin = plugin.main_thread().as_mut();

            plugin.changed();

            Ok(())
        });
    }
}

use crate::utils::data_from_array_buf;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
