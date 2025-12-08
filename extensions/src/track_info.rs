#![warn(missing_docs)]

//! Allows a plugin to query the host for information about the track it's in.
//!
//! See the [`TrackInfo`] type's documentation for a list of all the info a plugin can get about
//! the track it is currently in.

use crate::audio_ports::AudioPortType;
use bitflags::bitflags;
use clack_common::extensions::*;
use clack_common::utils::{Color, TRANSPARENT_COLOR};
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
///
/// # Example
///
/// ```
/// use clack_extensions::track_info::{HostTrackInfo, TrackInfo, TrackInfoBuffer};
/// use clack_plugin::prelude::*;
///
/// # fn test(host_track_info: HostTrackInfo, host_main_thread_handle: HostMainThreadHandle) {
/// let host_track_info: HostTrackInfo = /* ... */
/// # host_track_info;
/// let mut host_handle: HostMainThreadHandle = /* ... */
/// # host_main_thread_handle;
///
/// let mut buffer = TrackInfoBuffer::new();
/// let info: Option<TrackInfo> = host_track_info.get(&mut host_handle, &mut buffer);
///
/// println!("{:?}", info.unwrap().name())
/// # }
/// ```
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
        /// Whether the [`name`](TrackInfo::name) field of a [`TrackInfo`] is set.
        ///
        /// The [`TrackInfo`] type sets, checks and manages this flag automatically. Most users
        /// of the [`TrackInfo`] type should not need to use this flag directly.
        const HAS_TRACK_NAME = CLAP_TRACK_INFO_HAS_TRACK_NAME;
        /// Whether the [`color`](TrackInfo::color) field of a [`TrackInfo`] is set.
        ///
        /// The [`TrackInfo`] type sets, checks and manages this flag automatically. Most users
        /// of the [`TrackInfo`] type should not need to use this flag directly.
        const HAS_TRACK_COLOR = CLAP_TRACK_INFO_HAS_TRACK_COLOR;
        /// Whether the [`audio_port_type`](TrackInfo::audio_port_type) and [`audio_channel_count`](TrackInfo::audio_channel_count) fields of a [`TrackInfo`] are set.
        ///
        /// The [`TrackInfo`] type sets, checks and manages this flag automatically. Most users
        /// of the [`TrackInfo`] type should not need to use this flag directly.
        const HAS_AUDIO_CHANNEL = CLAP_TRACK_INFO_HAS_AUDIO_CHANNEL;

        /// Whether the plugin is located on a return track (sometimes referred to as Send or FX track).
        ///
        /// Plugins may initialize with settings appropriate for send/parallel processing, e.g. being
        /// set with 100% wet.
        const IS_FOR_RETURN_TRACK = CLAP_TRACK_INFO_IS_FOR_RETURN_TRACK;
        /// Whether the plugin is located on a bus track.
        ///
        /// Plugins may initialize with settings appropriate for bus processing.
        const IS_FOR_BUS = CLAP_TRACK_INFO_IS_FOR_BUS;

        /// Whether the plugin is located on the master track.
        ///
        /// Plugins may initialize with settings appropriate for channel processing.
        const IS_FOR_MASTER = CLAP_TRACK_INFO_IS_FOR_MASTER;
    }
}

/// Information about a track the plugin is on.
///
/// This structure provides the following information:
///
/// * The [`name`](TrackInfo::name) of the track;
/// * The [`color`](TrackInfo::color) of the track;
/// * The audio configuration of the track, i.e. its [`audio_port_type`](TrackInfo::audio_port_type)
///   and its [`audio_channel_count`](TrackInfo::audio_channel_count);
/// * [Flags](TrackInfo::flags) which indicate whether the track is a
///   [return track](TrackInfoFlags::IS_FOR_RETURN_TRACK), [bus track](TrackInfoFlags::IS_FOR_BUS),
///   or [master track](TrackInfoFlags::IS_FOR_MASTER).
///
/// Each of those fields can be set either with accessors (e.g. [`set_name`](TrackInfo::set_name)) or with
/// builder-pattern-style helpers (e.g. [`with_name`](TrackInfo::with_name)).
///
/// All the information mentioned above is optional, and may or may not be provided by the host
/// depending on context. The [`TrackInfo::new`] constructor creates an info structure with none of
/// the fields set.
///
/// This type is generic over two different lifetimes:
/// * The reference to the [`AudioPortType`] identifier string `'a`;
/// * The reference to the [`name`](TrackInfo::name) string buffer provided by the host `'n`.
pub struct TrackInfo<'a, 'n> {
    flags: TrackInfoFlags,
    name: Option<&'n [u8]>,
    color: Color,
    audio_channel_count: u32,
    audio_port_type: Option<AudioPortType<'a>>,
}

impl<'a, 'n> TrackInfo<'a, 'n> {
    /// Creates a new track information struct with none of the fields set.
    #[inline]
    pub const fn new() -> Self {
        Self {
            flags: TrackInfoFlags::empty(),
            name: None,
            color: TRANSPARENT_COLOR,
            audio_channel_count: 0,
            audio_port_type: None,
        }
    }

    /// Reads the track information from a raw, C-FFI compatible CLAP track info struct.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `audio_port_type` pointer is either NULL, or points to a valid
    /// NULL-terminated C string and remains valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_raw(raw: &'n clap_track_info) -> Self {
        let flags = TrackInfoFlags::from_bits_truncate(raw.flags);

        let (port_type, channel_count) = if flags.contains(TrackInfoFlags::HAS_AUDIO_CHANNEL) {
            (
                // SAFETY: upheld by caller
                unsafe { AudioPortType::from_raw(raw.audio_port_type) },
                raw.audio_channel_count,
            )
        } else {
            (None, 0)
        };

        let color = if flags.contains(TrackInfoFlags::HAS_TRACK_COLOR) {
            raw.color
        } else {
            TRANSPARENT_COLOR
        };

        Self {
            flags,
            name: flags
                .contains(TrackInfoFlags::HAS_TRACK_NAME)
                .then(|| data_from_array_buf(&raw.name)),
            color,
            audio_port_type: port_type,
            audio_channel_count: u32::try_from(channel_count).unwrap_or(0),
        }
    }

    /// Returns the set of flags applied to this track information.
    ///
    /// See the [`TrackInfoFlags`] type's documentation for a list of flags and their meanings.
    #[inline]
    pub const fn flags(&self) -> TrackInfoFlags {
        self.flags
    }

    /// Sets and replaces the flags applied to this track information.
    ///
    /// See the [`TrackInfoFlags`] type's documentation for a list of flags and their meanings.
    #[inline]
    pub const fn set_flags(&mut self, flags: TrackInfoFlags) {
        self.flags = flags;
    }

    /// Sets and replaces the flags applied to this track information.
    ///
    /// See the [`TrackInfoFlags`] type's documentation for a list of flags and their meanings.
    #[inline]
    pub const fn with_flags(mut self, flags: TrackInfoFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Returns the name of the track the plugin is on.
    ///
    /// If the [`HAS_TRACK_NAME`](TrackInfoFlags::HAS_TRACK_NAME) [flag](Self::flags) is unset,
    /// then this will always return `None`.
    #[inline]
    pub const fn name(&self) -> Option<&'n [u8]> {
        if !self.flags.contains(TrackInfoFlags::HAS_TRACK_NAME) {
            return None;
        }

        self.name
    }

    /// Sets (or unsets) the name of the track the plugin is on.
    ///
    /// This method automatically updates the [`HAS_TRACK_NAME`](TrackInfoFlags::HAS_TRACK_NAME) [flag](Self::flags) accordingly.
    #[inline]
    pub const fn set_name(&mut self, name: Option<&'n [u8]>) {
        self.set_flag(TrackInfoFlags::HAS_TRACK_NAME, name.is_some());
        self.name = name;
    }

    /// Sets (or unsets) the name of the track the plugin is on.
    ///
    /// This method automatically updates the [`HAS_TRACK_NAME`](TrackInfoFlags::HAS_TRACK_NAME) [flag](Self::flags) accordingly.
    #[inline]
    pub const fn with_name(mut self, name: Option<&'n [u8]>) -> Self {
        self.set_name(name);
        self
    }

    /// Returns the color of the track the plugin is on.
    ///
    /// If the [`HAS_TRACK_COLOR`](TrackInfoFlags::HAS_TRACK_COLOR) [flag](Self::flags) is unset,
    /// then this will always return `None`.
    #[inline]
    pub const fn color(&self) -> Option<Color> {
        if !self.flags.contains(TrackInfoFlags::HAS_TRACK_COLOR) {
            return None;
        }

        Some(self.color)
    }

    /// Sets (or unsets) the color of the track the plugin is on.
    ///
    /// This method automatically updates the [`HAS_TRACK_COLOR`](TrackInfoFlags::HAS_TRACK_COLOR) [flag](Self::flags) accordingly.
    #[inline]
    pub const fn set_color(&mut self, color: Option<Color>) {
        self.set_flag(TrackInfoFlags::HAS_TRACK_COLOR, color.is_some());

        self.color = match color {
            Some(color) => color,
            None => TRANSPARENT_COLOR,
        };
    }

    /// Sets (or unsets) the color of the track the plugin is on.
    ///
    /// This method automatically updates the [`HAS_TRACK_COLOR`](TrackInfoFlags::HAS_TRACK_COLOR) [flag](Self::flags) accordingly.
    #[inline]
    pub const fn with_color(mut self, color: Option<Color>) -> Self {
        self.set_color(color);
        self
    }

    /// Returns the number of audio channels of the track the plugin is on.
    ///
    /// If the [`HAS_AUDIO_CHANNEL`](TrackInfoFlags::HAS_AUDIO_CHANNEL) [flag](Self::flags) is unset,
    /// then this will always return `None`.
    #[inline]
    pub const fn audio_channel_count(&self) -> Option<u32> {
        if !self.flags.contains(TrackInfoFlags::HAS_AUDIO_CHANNEL) {
            return None;
        }

        Some(self.audio_channel_count)
    }

    /// Returns the port layout of the track the plugin is on, as an [`AudioPortType`].
    ///
    /// If the [`HAS_AUDIO_CHANNEL`](TrackInfoFlags::HAS_AUDIO_CHANNEL) [flag](Self::flags) is unset,
    /// then this will always return `None`.
    #[inline]
    pub const fn audio_port_type(&self) -> Option<AudioPortType<'a>> {
        if !self.flags.contains(TrackInfoFlags::HAS_AUDIO_CHANNEL) {
            return None;
        }

        self.audio_port_type
    }

    /// Sets (or unsets) the audio channel layout and count of the track the plugin is on.
    ///
    /// If the given `audio_port_type` is `None`, then the `channel_count` parameter is completely ignored.
    ///
    /// This method automatically updates the [`HAS_AUDIO_CHANNEL`](TrackInfoFlags::HAS_AUDIO_CHANNEL) [flag](Self::flags) accordingly.
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

    /// Sets (or unsets) the audio channel layout and count of the track the plugin is on.
    ///
    /// If the given `audio_port_type` is `None`, then the `channel_count` parameter is completely ignored.
    ///
    /// This method automatically updates the [`HAS_AUDIO_CHANNEL`](TrackInfoFlags::HAS_AUDIO_CHANNEL) [flag](Self::flags) accordingly.
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
    /// Creates a new track information struct with none of the fields set.
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
    use std::mem::MaybeUninit;

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

    /// A helper type that allows to safely write a [`TrackInfo`] to an uninitialized plugin-provided
    /// buffer.
    ///
    /// This type wraps a pointer to a plugin-provided, potentially uninitialized track info buffer,
    /// and exposes the [`set`](TrackInfoWriter::set) method to safely write into it.
    pub struct TrackInfoWriter<'buf, 'port_type> {
        buffer: *mut clap_track_info,
        _buffer: PhantomData<&'buf mut clap_track_info>,
        _audio_port_type: PhantomData<AudioPortType<'port_type>>,
        is_set: bool,
    }

    impl<'buf, 'port_type> TrackInfoWriter<'buf, 'port_type> {
        /// Wraps a given mutable reference to a potentially initialized C-FFI compatible buffer.
        pub const fn from_raw_buf(buffer: &'buf mut MaybeUninit<clap_track_info>) -> Self {
            // SAFETY: Coming from a &mut guarantees the pointer is valid for writes, non-null and aligned.
            unsafe { Self::from_raw(buffer.as_mut_ptr()) }
        }

        /// Wraps a given pointer to a C-FFI compatible buffer.
        ///
        /// # Safety
        ///
        /// Callers must ensure the pointer must be valid for writes for the lifetime of `'buf`. It
        /// must also be non-null and well-aligned.
        #[inline]
        pub const unsafe fn from_raw(ptr: *mut clap_track_info) -> Self {
            Self {
                buffer: ptr,
                _buffer: PhantomData,
                _audio_port_type: PhantomData,
                is_set: false,
            }
        }

        /// Writes the given `track_info` into the buffer this type wraps.
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
                    track_info.color().unwrap_or(TRANSPARENT_COLOR),
                );
                write(
                    &raw mut (*buf).audio_channel_count,
                    track_info
                        .audio_channel_count()
                        .unwrap_or(0)
                        .try_into()
                        .unwrap_or(i32::MAX),
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

    /// A buffer for hosts to write track information into.
    ///
    /// This is to be passed the [`HostTrackInfo::get`] method, which allows the host to write into
    /// it and to then retrieve a valid [`TrackInfo`] from it.
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
        /// Creates a new, empty track info buffer.
        #[inline]
        pub const fn new() -> Self {
            Self {
                inner: MaybeUninit::zeroed(),
            }
        }
    }

    impl HostTrackInfo {
        /// Request the host to write the current track information into the given buffer.
        ///
        /// If successful, a valid [`TrackInfo`] (referencing the given buffer) is returned.
        /// Otherwise, [`None`] is returned.
        pub fn get<'host, 'buffer>(
            &self,
            host: &'host mut HostMainThreadHandle,
            buffer: &'buffer mut TrackInfoBuffer,
        ) -> Option<TrackInfo<'host, 'buffer>> {
            let get = host.use_extension(&self.0).get?;

            // SAFETY: This type ensures the function pointer is valid.
            let success = unsafe { get(host.as_raw(), buffer.inner.as_mut_ptr()) };
            if !success {
                return None;
            }

            // SAFETY: Per the CLAP spec, success being set guarantees that the buffer is initialized.
            // Worst case, the host didn't actually initialize some (or all) of these, and everything is
            // zeroed, which is a valid bit pattern for all fields of the struct.
            let raw = unsafe { buffer.inner.assume_init_ref() };
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
    unsafe impl<P> ExtensionImplementation<P> for PluginTrackInfo
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
