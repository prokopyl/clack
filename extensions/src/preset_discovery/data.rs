use bitflags::bitflags;
use clack_common::utils::Timestamp;
use clap_sys::factory::preset_discovery::*;
use std::ffi::{CStr, c_char};

#[derive(Copy, Clone, Debug)]
pub struct FileType<'a> {
    pub name: &'a CStr,
    pub description: Option<&'a CStr>,
    pub file_extension: Option<&'a CStr>,
}

impl FileType<'_> {
    pub unsafe fn from_raw(raw: *const clap_preset_discovery_filetype) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        // SAFETY: TODO
        let raw = unsafe { raw.read() };

        // SAFETY: TODO
        unsafe {
            Some(Self {
                name: str_from_raw(raw.name)?,
                description: str_from_raw(raw.description),
                file_extension: str_from_raw(raw.file_extension),
            })
        }
    }

    #[inline]
    pub fn to_raw(&self) -> clap_preset_discovery_filetype {
        clap_preset_discovery_filetype {
            name: self.name.as_ptr(),
            description: str_to_raw(self.description),
            file_extension: str_to_raw(self.file_extension),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LocationData<'a> {
    pub name: &'a CStr,
    pub flags: Flags,
    pub location: Location<'a>,
}

impl LocationData<'_> {
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

    pub unsafe fn from_raw(raw: *const clap_preset_discovery_location) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        // SAFETY: TODO
        let raw = unsafe { raw.read() };

        // SAFETY: TODO
        unsafe {
            Some(Self {
                name: str_from_raw(raw.name)?,
                flags: Flags::from_bits_truncate(raw.flags),
                location: Location::from_raw(raw.kind, raw.location)?,
            })
        }
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Flags: u32 {
        const IS_FACTORY_CONTENT = CLAP_PRESET_DISCOVERY_IS_FACTORY_CONTENT;
        const IS_USER_CONTENT = CLAP_PRESET_DISCOVERY_IS_USER_CONTENT;
        const IS_DEMO_CONTENT = CLAP_PRESET_DISCOVERY_IS_DEMO_CONTENT;
        const IS_FAVORITE = CLAP_PRESET_DISCOVERY_IS_FAVORITE;
    }
}

impl Default for Flags {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Location<'a> {
    Plugin,
    File { path: &'a CStr },
}

impl<'a> Location<'a> {
    #[inline]
    pub fn to_raw(self) -> (clap_preset_discovery_location_kind, *const c_char) {
        match self {
            Location::Plugin => (CLAP_PRESET_DISCOVERY_LOCATION_PLUGIN, core::ptr::null()),
            Location::File { path } => (CLAP_PRESET_DISCOVERY_LOCATION_FILE, path.as_ptr()),
        }
    }

    #[inline]
    pub unsafe fn from_raw(
        kind: clap_preset_discovery_location_kind,
        path: *const c_char,
    ) -> Option<Self> {
        match kind {
            CLAP_PRESET_DISCOVERY_LOCATION_PLUGIN => Some(Location::Plugin),
            CLAP_PRESET_DISCOVERY_LOCATION_FILE if !path.is_null() => Some(Location::File {
                // SAFETY: TODO
                path: unsafe { CStr::from_ptr(path) },
            }),
            _ => None,
        }
    }

    #[inline]
    pub fn file_path(&self) -> Option<&'a CStr> {
        match self {
            Location::Plugin => None,
            Location::File { path } => Some(*path),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Soundpack<'a> {
    pub flags: Flags,
    pub id: &'a CStr,
    pub name: &'a CStr,
    pub description: Option<&'a CStr>,
    pub homepage_url: Option<&'a CStr>,
    pub vendor: Option<&'a CStr>,
    pub image_path: Option<&'a CStr>,
    pub release_timestamp: Option<Timestamp>,
}

impl Soundpack<'_> {
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

    pub unsafe fn from_raw(raw: *const clap_preset_discovery_soundpack) -> Option<Self> {
        if raw.is_null() {
            return None;
        }

        // SAFETY: TODO
        let raw = unsafe { raw.read() };

        // SAFETY: TODO
        unsafe {
            Some(Self {
                flags: Flags::from_bits_truncate(raw.flags),
                id: str_from_raw(raw.id)?,
                name: str_from_raw(raw.name)?,
                description: str_from_raw(raw.description),
                homepage_url: str_from_raw(raw.homepage_url),
                vendor: str_from_raw(raw.vendor),
                image_path: str_from_raw(raw.image_path),
                release_timestamp: Timestamp::from_raw(raw.release_timestamp),
            })
        }
    }
}

#[inline]
unsafe fn str_from_raw<'a>(raw: *const c_char) -> Option<&'a CStr> {
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
fn str_to_raw(str: Option<&CStr>) -> *const c_char {
    match str {
        None => std::ptr::null(),
        Some(str) => str.as_ptr(),
    }
}
