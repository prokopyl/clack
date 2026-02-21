#![warn(missing_docs)]
//! Various data types that can are used to locate or categorize presets.

use bitflags::bitflags;
use clack_common::utils::Timestamp;
use clap_sys::factory::preset_discovery::*;
use std::ffi::{CStr, c_char};

/// A type of file the host should match for when searching preset directory locations.
#[derive(Copy, Clone, Debug)]
pub struct FileType<'a> {
    /// The name of the file type.
    pub name: &'a CStr,
    /// An optional description of the file type.
    pub description: Option<&'a CStr>,
    /// The extension of that file (excluding the '.').
    ///
    /// If this is `None` or an empty string, then every file will be matched.
    pub file_extension: Option<&'a CStr>,
}

impl FileType<'_> {
    /// Creates a [`FileType`] from its raw, C-FFI compatible representation.
    ///
    /// If the `name` string pointer within the given struct is `NULL`, this returns [`None`].
    ///
    /// # Safety
    ///
    /// Either of the contained pointers can be `NULL`, in which case the following requirements do
    /// not apply.
    ///
    /// Unless they are NULL, all the contained string pointers must point to valid C strings, and
    /// remain valid for the duration of the `'a` lifetime.
    ///
    /// See the documentation of [`CStr::from_ptr`] for the exhaustive list of safety requirements
    /// for each of those pointers.
    pub const unsafe fn from_raw(raw: clap_preset_discovery_filetype) -> Option<Self> {
        // SAFETY: All C strings are either NULL or valid as upheld by the caller
        unsafe {
            Some(Self {
                name: match str_from_raw(raw.name) {
                    Some(name) => name,
                    None => return None,
                },
                description: str_from_raw(raw.description),
                file_extension: str_from_raw(raw.file_extension),
            })
        }
    }

    /// Creates a [`FileType`] from a pointer to its raw, C-FFI compatible representation.
    ///
    /// If either the given `ptr`, or the `name` string pointer are `NULL`, this returns [`None`].
    ///
    /// This function is an alternative to the [`from_raw`](Self::from_raw) method, except it allows to read
    /// directly from a raw pointer without actually borrowing it, which may be slightly safer in
    /// some cases.
    ///
    /// # Safety
    ///
    /// The given `ptr` must be well-aligned and valid for reads.
    /// On top of that, the [`clap_preset_discovery_filetype`] value it points to must also satisfy the
    /// safety requirements of the [`from_raw`](Self::from_raw) method.
    #[inline]
    pub const unsafe fn from_raw_ptr(ptr: *const clap_preset_discovery_filetype) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: pointer is guaranteed to be valid for reads by caller
        let plugin_id = unsafe { ptr.read() };
        // SAFETY: invariants are upheld by callers
        unsafe { Self::from_raw(plugin_id) }
    }

    /// Returns the raw, C-FFI compatible representation of this ID.
    ///
    /// The pointers contained in this struct are valid for the `'a` lifetime.
    #[inline]
    pub const fn to_raw(&self) -> clap_preset_discovery_filetype {
        clap_preset_discovery_filetype {
            name: self.name.as_ptr(),
            description: str_to_raw(self.description),
            file_extension: str_to_raw(self.file_extension),
        }
    }
}

/// Information about a preset location.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LocationInfo<'a> {
    /// A user-friendly name for this location.
    pub name: &'a CStr,
    /// Flags describing extra information about the presets at the location.
    /// These can be overridden in a per-preset basis. They are used as a fallback only if the
    /// preset didn't specify them.
    pub flags: Flags,
    /// The actual preset location.
    pub location: Location<'a>,
}

impl LocationInfo<'_> {
    /// Returns the raw, C-FFI compatible representation of this Location information.
    ///
    /// The pointers contained in this struct are valid for the `'a` lifetime.
    #[inline]
    pub fn to_raw(self) -> clap_preset_discovery_location {
        let (kind, location) = self.location.to_raw();

        clap_preset_discovery_location {
            flags: self.flags.bits(),
            name: self.name.as_ptr(),
            kind,
            location,
        }
    }

    /// Creates a [`LocationInfo`] from its raw, C-FFI compatible representation.
    ///
    /// If the `location` pointer is `NULL` when the `kind` is `CLAP_PRESET_DISCOVERY_LOCATION_FILE`,
    /// then this returns [`None`].
    ///
    /// # Safety
    ///
    /// Either of the contained pointers can be `NULL`, in which case the following requirements do
    /// not apply.
    ///
    /// Unless they are NULL, all the contained string pointers must point to valid C strings, and
    /// remain valid for the duration of the `'a` lifetime.
    ///
    /// See the documentation of [`CStr::from_ptr`] for the exhaustive list of safety requirements
    /// for each of those pointers.
    pub const unsafe fn from_raw(raw: clap_preset_discovery_location) -> Option<Self> {
        // SAFETY: All C strings are either NULL or valid as upheld by the caller
        unsafe {
            Some(Self {
                name: match str_from_raw(raw.name) {
                    Some(name) => name,
                    None => c"",
                },
                flags: Flags::from_bits_truncate(raw.flags),
                location: match Location::from_raw(raw.kind, raw.location) {
                    Some(location) => location,
                    None => return None,
                },
            })
        }
    }

    /// Creates a [`FileType`] from a pointer to its raw, C-FFI compatible representation.
    ///
    /// If either the given `ptr` is `NULL`, or if the `location` pointer is `NULL` when the `kind`
    /// is `CLAP_PRESET_DISCOVERY_LOCATION_FILE`, then this returns [`None`].
    ///
    /// This function is an alternative to the [`from_raw`](Self::from_raw) method, except it allows to read
    /// directly from a raw pointer without actually borrowing it, which may be slightly safer in
    /// some cases.
    ///
    /// # Safety
    ///
    /// The given `ptr` must be well-aligned and valid for reads.
    /// On top of that, the `clap_preset_discovery_location` value it points to must also satisfy the
    /// safety requirements of the [`from_raw`](Self::from_raw) method.
    pub const unsafe fn from_raw_ptr(raw: *const clap_preset_discovery_location) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        // SAFETY: pointer is guaranteed to be valid for reads by caller
        let raw = unsafe { raw.read() };

        // SAFETY: invariants are upheld by callers
        unsafe { Self::from_raw(raw) }
    }
}

bitflags! {
    /// A set of flags representing extra information about a preset.
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Flags: u32 {
        /// This is a "factory" or sound-pack preset.
        const IS_FACTORY_CONTENT = CLAP_PRESET_DISCOVERY_IS_FACTORY_CONTENT;

        /// This preset was created by the user.
        const IS_USER_CONTENT = CLAP_PRESET_DISCOVERY_IS_USER_CONTENT;

        /// This is a demo preset.
        const IS_DEMO_CONTENT = CLAP_PRESET_DISCOVERY_IS_DEMO_CONTENT;

        /// This preset was favorited by the user.
        const IS_FAVORITE = CLAP_PRESET_DISCOVERY_IS_FAVORITE;
    }
}

impl Default for Flags {
    /// Returns empty flags.
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

/// A location that can contain presets.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Location<'a> {
    /// The plugin itself.
    ///
    /// This is used to mean that the plugin has presets included in its own entry file.
    Plugin,
    /// A file or directory.
    File {
        /// The path of the file or directory.
        path: &'a CStr,
    },
}

impl<'a> Location<'a> {
    /// Returns this location as a `location_kind` tag along with an optional file path string.
    ///
    /// If this is [`Location::Plugin`], then the returned pointer is `NULL`.
    /// If this is [`Location::File`], then the returned pointer contains the file path and is
    /// valid for `'a`.
    #[inline]
    pub const fn to_raw(self) -> (clap_preset_discovery_location_kind, *const c_char) {
        match self {
            Location::Plugin => (CLAP_PRESET_DISCOVERY_LOCATION_PLUGIN, core::ptr::null()),
            Location::File { path } => (CLAP_PRESET_DISCOVERY_LOCATION_FILE, path.as_ptr()),
        }
    }

    /// Creates a [`Location`] from its raw, C-FFI compatible representation, consisting of a
    /// `location_kind` tag along its optional `path`.
    ///
    /// This function returns [`None`] if the given `kind` is unknown, or if it is
    /// `CLAP_PRESET_DISCOVERY_LOCATION_FILE` while the given `path` is `NULL`.
    ///
    /// # Safety
    ///
    /// If the given `kind` is `CLAP_PRESET_DISCOVERY_LOCATION_FILE`, then the given `path` must
    /// be either `NULL` or point to a valid, null-terminated C string, and remain valid for the
    /// `'a` lifetime.
    ///
    /// See the documentation of [`CStr::from_ptr`] for the exhaustive list of safety requirements
    /// for the `path` pointer.
    ///
    /// For any other value of `kind` (including unknown ones), this function is always safe to
    /// call, as the `path` pointer is simply discarded.
    #[inline]
    pub const unsafe fn from_raw(
        kind: clap_preset_discovery_location_kind,
        path: *const c_char,
    ) -> Option<Self> {
        match kind {
            CLAP_PRESET_DISCOVERY_LOCATION_PLUGIN => Some(Location::Plugin),
            CLAP_PRESET_DISCOVERY_LOCATION_FILE if !path.is_null() => Some(Location::File {
                // SAFETY: Strings are either NULL or valid as upheld by the caller.
                // We just checked above that the pointer is not NULL.
                path: unsafe { CStr::from_ptr(path) },
            }),
            _ => None,
        }
    }

    /// Returns the file path of this [`Location::File`].
    ///
    /// This returns [`None`] if this location is actually [`Location::Plugin`].
    #[inline]
    pub const fn file_path(&self) -> Option<&'a CStr> {
        match self {
            Location::Plugin => None,
            Location::File { path } => Some(*path),
        }
    }
}

/// Information about an installed Sound Pack.
#[derive(Copy, Clone, Debug)]
pub struct Soundpack<'a> {
    /// Flags describing extra information about the presets in this soundpack.
    pub flags: Flags,
    /// The unique identifier of this sound pack.
    ///
    /// This will be used by presets to indicate which sound pack(s) they belong to.
    pub id: &'a CStr,
    /// The display name for this sound pack.
    pub name: &'a CStr,
    /// An extended description about this sound pack. Optional.
    pub description: Option<&'a CStr>,
    /// The URL of this sound pack's homepage, if it has one. Optional.
    pub homepage_url: Option<&'a CStr>,
    /// The vendor of this sound pack. Optional.
    pub vendor: Option<&'a CStr>,
    /// The on-disk path of an illustration image of this sound pack. Optional.
    pub image_path: Option<&'a CStr>,
    /// The time at which this sound pack was released. This can be [`None`] if unavailable.
    pub release_timestamp: Option<Timestamp>,
}

impl<'a> Soundpack<'a> {
    /// Returns the raw, C-FFI compatible representation of this Location information.
    ///
    /// The pointers contained in this struct are valid for the `'a` lifetime.
    pub fn to_raw(self) -> clap_preset_discovery_soundpack {
        clap_preset_discovery_soundpack {
            flags: self.flags.bits(),
            id: self.id.as_ptr(),
            name: self.name.as_ptr(),
            description: str_to_raw(self.description),
            homepage_url: str_to_raw(self.homepage_url),
            vendor: str_to_raw(self.vendor),
            image_path: str_to_raw(self.image_path),
            release_timestamp: Timestamp::optional_to_raw(self.release_timestamp),
        }
    }

    /// Creates a [`Soundpack`] from its raw, C-FFI compatible representation.
    ///
    /// If any of the non-optional pointers are `NULL`, then this returns [`None`].
    ///
    /// # Safety
    ///
    /// Any of the contained pointers can be `NULL`, in which case the following requirements do
    /// not apply.
    ///
    /// Unless they are NULL, all the contained string pointers must point to valid C strings, and
    /// remain valid for the duration of the `'a` lifetime.
    ///
    /// See the documentation of [`CStr::from_ptr`] for the exhaustive list of safety requirements
    /// for each of those pointers.
    pub const unsafe fn from_raw(raw: clap_preset_discovery_soundpack) -> Option<Self> {
        // SAFETY: All C strings are either NULL or valid as upheld by the caller
        unsafe {
            Some(Self {
                flags: Flags::from_bits_truncate(raw.flags),
                id: match str_from_raw(raw.id) {
                    Some(id) => id,
                    None => return None,
                },
                name: match str_from_raw(raw.name) {
                    Some(name) => name,
                    None => return None,
                },
                description: str_from_raw(raw.description),
                homepage_url: str_from_raw(raw.homepage_url),
                vendor: str_from_raw(raw.vendor),
                image_path: str_from_raw(raw.image_path),
                release_timestamp: Timestamp::from_raw(raw.release_timestamp),
            })
        }
    }

    /// Creates a [`Soundpack`] from a pointer to its raw, C-FFI compatible representation.
    ///
    /// If either the given `ptr` is `NULL`, or if any of the non-optional pointers are `NULL`,
    /// then this returns [`None`].
    ///
    /// This function is an alternative to the [`from_raw`](Self::from_raw) method, except it allows to read
    /// directly from a raw pointer without actually borrowing it, which may be slightly safer in
    /// some cases.
    ///
    /// # Safety
    ///
    /// The given `ptr` must be well-aligned and valid for reads.
    /// On top of that, the `clap_preset_discovery_soundpack` value it points to must also satisfy the
    /// safety requirements of the [`from_raw`](Self::from_raw) method.
    pub const unsafe fn from_raw_ptr(raw: *const clap_preset_discovery_soundpack) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        // SAFETY: pointer is guaranteed to be valid for reads by caller
        let raw = unsafe { raw.read() };

        // SAFETY: invariants are upheld by callers
        unsafe { Self::from_raw(raw) }
    }
}

/// # Safety
///
/// Same as [`CStr::from_ptr`], except the pointer can be null.
#[inline]
const unsafe fn str_from_raw<'a>(raw: *const c_char) -> Option<&'a CStr> {
    if raw.is_null() {
        return None;
    }

    let str = CStr::from_ptr(raw);

    if str.is_empty() {
        return None;
    }

    Some(str)
}

#[inline]
const fn str_to_raw(str: Option<&CStr>) -> *const c_char {
    match str {
        None => std::ptr::null(),
        Some(str) => str.as_ptr(),
    }
}
