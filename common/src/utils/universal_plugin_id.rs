#![deny(missing_docs)]

use clap_sys::universal_plugin_id::clap_universal_plugin_id;
use core::ffi::CStr;

/// Known ABI types that can be used in [`UniversalPluginId`]s.
///
/// This is non-exhaustive, as other plugin ID types may be added in the future.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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
    /// If the ABI type string doesn't match a known [`UniversalPluginAbi`] variant, this returns
    /// [`None`].
    ///
    /// This comparison is case-sensitive.
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

/// A unique identifier that works across plugin APIs.
///
/// This is a ([`abi`](Self::abi), [`id`](Self::id)) pair of borrowed strings that live for the `'a`
/// lifetime.
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
    /// Creates a [`UniversalPluginId`] from its raw, C-FFI compatible representation.
    ///
    /// If either of the string pointers within the given struct are `NULL`, this returns [`None`].
    ///
    /// # Safety
    ///
    /// Either of the contained pointers can be `NULL`, in which case the following requirements do
    /// not apply.
    ///
    /// Both pointers must point to valid C strings, and remain valid for the duration of the `'a` lifetime.
    ///
    /// See the documentation of [`CStr::from_ptr`] for the exhaustive list of safety requirements.
    #[inline]
    pub const unsafe fn from_raw(raw: clap_universal_plugin_id) -> Option<Self> {
        Some(Self {
            abi: if raw.abi.is_null() {
                return None;
            } else {
                // SAFETY: pointer is guaranteed to be valid by caller
                unsafe { CStr::from_ptr(raw.abi) }
            },
            id: if raw.id.is_null() {
                return None;
            } else {
                // SAFETY: pointer is guaranteed to be valid by caller
                unsafe { CStr::from_ptr(raw.id) }
            },
        })
    }

    /// Creates a [`UniversalPluginId`] from a pointer to its raw, C-FFI compatible representation.
    ///
    /// If the given `ptr`, or either of the string pointers within the pointed struct are `NULL`, this returns [`None`].
    ///
    /// This function is a version of the [`from_raw`](Self::from_raw) method, except it allows to read
    /// directly from a raw pointer without actually borrowing it, which may be slightly safer.
    ///
    /// # Safety
    ///
    /// The given `ptr` must be well-aligned and valid for that.
    /// On top of that, the [`clap_universal_plugin_id`] value it points to must also satisfy the
    /// safety requirements of the [`from_raw`](Self::from_raw) method.
    #[inline]
    pub const unsafe fn from_raw_ptr(ptr: *const clap_universal_plugin_id) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        // SAFETY: pointer is guaranteed to be valid for reads by caller
        let plugin_id = unsafe { ptr.read() };
        Self::from_raw(plugin_id)
    }

    /// Creates a [`UniversalPluginId`] for a CLAP plugin with a given `id`.
    #[inline]
    pub const fn clap(id: &'a CStr) -> Self {
        Self { abi: c"clap", id }
    }

    /// Returns the [`UniversalPluginAbi`] matching this ID's plugin ABI.
    ///
    /// If the ABI type string doesn't match a known [`UniversalPluginAbi`] variant, this returns
    /// [`None`].
    #[inline]
    pub const fn abi(&self) -> Option<UniversalPluginAbi> {
        UniversalPluginAbi::from_abi_str(self.abi)
    }

    /// Returns a raw, C-FFI compatible representation of this ID.
    ///
    /// The pointers contained in this struct are valid for the `'a` lifetime.
    #[inline]
    pub const fn to_raw(&self) -> clap_universal_plugin_id {
        clap_universal_plugin_id {
            id: self.id.as_ptr(),
            abi: self.abi.as_ptr(),
        }
    }
}
