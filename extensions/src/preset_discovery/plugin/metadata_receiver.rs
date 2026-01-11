use crate::preset_discovery::preset_data::Flags;
use crate::utils::cstr_to_nullable_ptr;
use clack_common::utils::{Timestamp, UniversalPluginId};
use clap_sys::factory::preset_discovery::clap_preset_discovery_metadata_receiver;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// A lightweight borrowed handle to a host-provided metadata receiver.
///
/// See the [`ProviderImpl::get_metadata`](super::provider::ProviderImpl::get_metadata) documentation for a usage example.
#[repr(C)]
pub struct MetadataReceiver<'a> {
    inner: clap_preset_discovery_metadata_receiver,
    // Raw pointer is here to make sure this is !Send !Sync
    lifetime: PhantomData<(&'a clap_preset_discovery_metadata_receiver, *const ())>,
}

impl MetadataReceiver<'_> {
    /// # Safety
    ///
    /// Pointer must be valid for 'a, as well as its contents
    pub(crate) unsafe fn from_raw<'a>(
        raw: *const clap_preset_discovery_metadata_receiver,
    ) -> &'a mut Self {
        // SAFETY: This is safe to transmute as it's repr(C) and has the same memory representation
        // Other safety invariants are upheld by the caller
        unsafe { &mut *(raw as *mut Self) }
    }

    /// Report to the host that an error occurred while reading metadata from a file.
    ///
    /// `error_code` is the operating system error, as returned by e.g. [`std::io::Error::raw_os_error`], if applicable.
    /// If not applicable, it should be set to a non-error value, e.g. 0 on Unix and Windows.
    ///
    /// Note that the handler for [`get_metadata`](super::provider::ProviderImpl::get_metadata) already
    /// calls this method when `Err` was returned, extracting the OS error code and formatting the error message.
    /// In that case, it is redundant to call this method when an error has occurred.
    #[inline]
    pub fn on_error(&mut self, error_code: i32, error_message: Option<&CStr>) {
        if let Some(on_error) = self.inner.on_error {
            // SAFETY: This type guarantees inner is valid. String pointers are valid as they come from references.
            unsafe { on_error(&self.inner, error_code, cstr_to_nullable_ptr(error_message)) }
        }
    }

    /// Begins describing a new preset.
    ///
    /// All other methods on this receiver called after this will apply to this preset.
    /// `name` is the user-friendly display name of this preset.
    ///
    /// `load_key` is a machine friendly unique string used to load the preset inside the container via
    /// the preset-load plug-in extension. It can just be the subpath if that's what
    /// the plugin wants, but it could also be some other unique ID like a database primary key or a
    /// binary offset. Its use is entirely up to the plug-in, its contents must be treated as opaque
    /// by the host.
    #[inline]
    pub fn begin_preset(
        &mut self,
        name: Option<&CStr>,
        load_key: Option<&CStr>,
    ) -> Result<&mut Self, MetadataReceiverError> {
        let Some(begin_preset) = self.inner.begin_preset else {
            return Err(MetadataReceiverError { _inner: () });
        };

        // SAFETY: This type guarantees inner is valid. String pointers are valid as they come from references.
        let success = unsafe {
            begin_preset(
                &self.inner,
                cstr_to_nullable_ptr(name),
                cstr_to_nullable_ptr(load_key),
            )
        };

        if success {
            Ok(self)
        } else {
            Err(MetadataReceiverError { _inner: () })
        }
    }

    /// Adds the ID of a plugin the current preset can be used with.
    #[inline]
    pub fn add_plugin_id(&mut self, plugin_id: UniversalPluginId) -> &mut Self {
        if let Some(add_plugin_id) = self.inner.add_plugin_id {
            let plugin_id = plugin_id.to_raw();
            // SAFETY: This type guarantees inner is valid. String pointers are valid as they come from references.
            unsafe { add_plugin_id(&self.inner, &plugin_id) }
        }
        self
    }

    /// Sets the sound pack to which the current preset belongs to.
    #[inline]
    pub fn set_soundpack_id(&mut self, soundpack_id: &CStr) -> &mut Self {
        if let Some(set_soundpack_id) = self.inner.set_soundpack_id {
            // SAFETY: This type guarantees inner is valid. String pointers are valid as they come from references.
            unsafe { set_soundpack_id(&self.inner, soundpack_id.as_ptr()) }
        }
        self
    }

    /// Sets flags specific to the current preset.
    #[inline]
    pub fn set_flags(&mut self, flags: Flags) -> &mut Self {
        if let Some(set_flags) = self.inner.set_flags {
            // SAFETY: This type guarantees inner is valid.
            unsafe { set_flags(&self.inner, flags.bits()) }
        }
        self
    }

    /// Adds a creator's name to the current preset.
    #[inline]
    pub fn add_creator(&mut self, creator: &CStr) -> &mut Self {
        if let Some(add_creator) = self.inner.add_creator {
            // SAFETY: This type guarantees inner is valid. String pointers are valid as they come from references.
            unsafe { add_creator(&self.inner, creator.as_ptr()) }
        }
        self
    }

    /// Sets the description the current preset.
    #[inline]
    pub fn set_description(&mut self, description: &CStr) -> &mut Self {
        if let Some(set_description) = self.inner.set_description {
            // SAFETY: This type guarantees inner is valid. String pointers are valid as they come from references.
            unsafe { set_description(&self.inner, description.as_ptr()) }
        }
        self
    }

    /// Sets the creation and last modification times of the current presets.
    ///
    /// If one of these is not known, it may be set to [`None`].
    #[inline]
    pub fn set_timestamps(
        &mut self,
        creation_time: Option<Timestamp>,
        modified_time: Option<Timestamp>,
    ) -> &mut Self {
        if let Some(set_timestamps) = self.inner.set_timestamps {
            // SAFETY: This type guarantees inner is valid.
            unsafe {
                set_timestamps(
                    &self.inner,
                    Timestamp::optional_to_raw(creation_time),
                    Timestamp::optional_to_raw(modified_time),
                )
            }
        }
        self
    }

    /// Adds a feature to the current preset.
    #[inline]
    pub fn add_feature(&mut self, feature: &CStr) -> &mut Self {
        if let Some(add_feature) = self.inner.add_feature {
            // SAFETY: This type guarantees inner is valid. String pointers are valid as they come from references.
            unsafe { add_feature(&self.inner, feature.as_ptr()) }
        }
        self
    }

    /// Adds extra metadata information to the current preset.
    #[inline]
    pub fn add_extra_info(&mut self, key: &CStr, value: &CStr) -> &mut Self {
        if let Some(add_extra_info) = self.inner.add_extra_info {
            // SAFETY: This type guarantees inner is valid. String pointers are valid as they come from references.
            unsafe { add_extra_info(&self.inner, key.as_ptr(), value.as_ptr()) }
        }
        self
    }
}

/// Error that can occur when using a [`MetadataReceiver`].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct MetadataReceiverError {
    _inner: (),
}

impl Display for MetadataReceiverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to begin a new preset in metadata receiver")
    }
}

impl Error for MetadataReceiverError {}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_any!(MetadataReceiver<'static>: Send, Sync);
}
