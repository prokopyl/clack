//! This module implements the VST3 plugin info extension of the clap-wrapper project.
//! Using these extensions, we can tell the wrapper how to advertise our CLAP plugins as VST3.

mod sys;
use sys::*;

#[cfg(feature = "clack-host")]
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

use bitflags::bitflags;
use clack_common::extensions::{Extension, PluginExtensionSide, RawExtension};
use clack_common::factory::{Factory, RawFactoryPointer};
use core::ffi::CStr;
use core::marker::PhantomData;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PluginInfoAsVST3<'a> {
    inner: clap_plugin_info_as_vst3,
    _lifetime: PhantomData<&'a CStr>,
}

// SAFETY: everything here is read-only
unsafe impl Send for PluginInfoAsVST3<'_> {}
// SAFETY: everything here is read-only
unsafe impl Sync for PluginInfoAsVST3<'_> {}

impl<'a> PluginInfoAsVST3<'a> {
    #[inline]
    pub const fn new(
        vendor: Option<&'a CStr>,
        component_id: Option<&'a [u8; 16]>,
        features: Option<&'a CStr>,
    ) -> Self {
        Self {
            _lifetime: PhantomData,
            inner: clap_plugin_info_as_vst3 {
                vendor: match vendor {
                    Some(v) => v.as_ptr(),
                    None => core::ptr::null(),
                },
                features: match features {
                    Some(v) => v.as_ptr(),
                    None => core::ptr::null(),
                },
                component_id: match component_id {
                    Some(v) => v,
                    None => core::ptr::null(),
                },
            },
        }
    }

    pub const fn vendor(&self) -> Option<&'a CStr> {
        if self.inner.vendor.is_null() {
            return None;
        }
        // SAFETY: this type enforces all the needed requirements
        unsafe { Some(CStr::from_ptr(self.inner.vendor)) }
    }

    pub const fn component_id(&self) -> Option<&'a [u8; 16]> {
        if self.inner.component_id.is_null() {
            return None;
        }

        // SAFETY: this type enforces all the needed requirements
        Some(unsafe { &*self.inner.component_id })
    }

    pub const fn features(&self) -> Option<&'a CStr> {
        if self.inner.features.is_null() {
            return None;
        }

        // SAFETY: this type enforces all the needed requirements
        unsafe { Some(CStr::from_ptr(self.inner.features)) }
    }

    #[inline]
    pub const fn as_raw(&self) -> &clap_plugin_info_as_vst3 {
        &self.inner
    }

    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - The given `raw` pointer is non-null, points to a well-aligned `clap_plugin_info_as_vst3`
    ///   instance that is valid for reads for the `'a` lifetime
    /// - Each field in `clap_plugin_info_as_vst3` is either null, or:
    ///   - for its `vendor` and `features` fields: must point to a null-terminated C string that is valid for reads
    ///   - for its `component_if` field: must point to a byte array of the correct size that is valid for reads
    pub const unsafe fn from_raw(raw: *const clap_plugin_info_as_vst3) -> &'a Self {
        // SAFETY:
        // - The caller upholds all conditions that `raw` is valid and points to a clap_plugin_info_as_vst3 struct with valid contents
        // - This type is #[repr(C)] and has the same memory layout as clap_plugin_info_as_vst3, so casting between the two is valid
        unsafe { &*(raw as *const Self) }
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginAsVST3(RawExtension<PluginExtensionSide, clap_plugin_as_vst3>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginAsVST3 {
    const IDENTIFIERS: &[&CStr] = &[CLAP_PLUGIN_AS_VST3];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SupportedNoteExpressions: u32 {
        const AS_VST3_NOTE_EXPRESSION_VOLUME = 1 << 0;
        const AS_VST3_NOTE_EXPRESSION_PAN = 1 << 1;
        const AS_VST3_NOTE_EXPRESSION_TUNING = 1 << 2;
        const AS_VST3_NOTE_EXPRESSION_VIBRATO = 1 << 3;
        const AS_VST3_NOTE_EXPRESSION_EXPRESSION = 1 << 4;
        const AS_VST3_NOTE_EXPRESSION_BRIGHTNESS = 1 << 5;
        const AS_VST3_NOTE_EXPRESSION_PRESSURE = 1 << 6;
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginAsVst3Factory<'a>(RawFactoryPointer<'a, clap_plugin_factory_as_vst3>);

// SAFETY: PluginFactoryWrapper is #[repr(C)] with clap_plugin_factory_as_vst3 as its first field, and matches
// CLAP_PLUGIN_FACTORY_INFO_VST3.
unsafe impl<'a> Factory<'a> for PluginAsVst3Factory<'a> {
    const IDENTIFIERS: &'static [&'static CStr] = &[CLAP_PLUGIN_FACTORY_INFO_VST3];
    type Raw = clap_plugin_factory_as_vst3;

    #[inline]
    unsafe fn from_raw(raw: RawFactoryPointer<'a, Self::Raw>) -> Self {
        Self(raw)
    }
}
