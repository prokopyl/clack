#![deny(missing_docs)]

use clap_sys::universal_plugin_id::clap_universal_plugin_id;
use core::ffi::CStr;

/// Known ABI types that can be used in [`UniversalPluginId`]s.
///
/// This is non-exhaustive, as other plugin ID types may be added in the future.
#[derive(Copy, Clone)]
#[non_exhaustive]
pub enum UniversalPluginAbi {
    /// The CLAP ABI.
    Clap,
    /// The VST3 ABI.
    VST3,
    /// The VST2 ABI.
    VST2,
    /// The AU ABI.
    AU,
}

impl UniversalPluginAbi {
    /// Returns the ABI string matching the given ABI type.
    ///
    /// This constant can then be used in the [`UniversalPluginId::abi`] field.
    #[inline]
    pub const fn abi_str(&self) -> &'static CStr {
        match self {
            UniversalPluginAbi::Clap => c"clap",
            UniversalPluginAbi::VST3 => c"vst3",
            UniversalPluginAbi::VST2 => c"vst2",
            UniversalPluginAbi::AU => c"au",
        }
    }

    /// Returns the ABI type matching the given ABI string.
    ///
    /// If
    #[inline]
    pub const fn from_abi_str(abi: &CStr) -> Option<Self> {
        match abi.to_bytes() {
            b"clap" => Some(UniversalPluginAbi::Clap),
            b"vst3" => Some(UniversalPluginAbi::VST3),
            b"vst2" => Some(UniversalPluginAbi::VST2),
            b"au" => Some(UniversalPluginAbi::AU),
            _ => None,
        }
    }
}

/// A unique identifier that works across plugin
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct UniversalPluginId<'a> {
    /// The ABI string.
    ///
    /// It can be e.g. "clap", "vst2", "vst3", "au", etc.
    ///
    /// See [`UniversalPluginAbi::abi_str`] for known values this field can take.
    pub abi: &'a CStr,
    /// The ID string of the plugin.
    ///
    /// This is formatted differently depending on the ABI type:
    ///
    /// * **CLAP**: Use the plugin ID (e.g. `"com.u-he.diva"`)
    /// * **AU**: Format the string as "type:subt:manu" (e.g. `"aumu:SgXT:VmbA"`)
    /// * **VST2**: Format the ID as a signed 32-bit integer (e.g. `"-4382976"`)
    /// * **VST3**: Format the ID as a hyphenated UUID (e.g. `"123e4567-e89b-12d3-a456-426614174000"`)
    pub id: &'a CStr,
}

impl<'a> UniversalPluginId<'a> {
    pub const unsafe fn from_raw_ptr(ptr: *const clap_universal_plugin_id) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: TODO
        let plugin_id = unsafe { ptr.read() };
        Self::from_raw(plugin_id)
    }

    pub const unsafe fn from_raw(raw: clap_universal_plugin_id) -> Option<Self> {
        Some(Self {
            abi: if raw.abi.is_null() {
                return None;
            } else {
                // SAFETY: TODO
                unsafe { CStr::from_ptr(raw.abi) }
            },
            id: if raw.id.is_null() {
                return None;
            } else {
                // SAFETY: TODO
                unsafe { CStr::from_ptr(raw.id) }
            },
        })
    }

    #[inline]
    pub const fn clap(id: &'a CStr) -> Self {
        Self { abi: c"clap", id }
    }

    #[inline]
    pub const fn abi(&self) -> Option<UniversalPluginAbi> {
        UniversalPluginAbi::from_abi_str(self.abi)
    }

    #[inline]
    pub const fn to_raw(&self) -> clap_universal_plugin_id {
        clap_universal_plugin_id {
            id: self.id.as_ptr(),
            abi: self.abi.as_ptr(),
        }
    }
}
