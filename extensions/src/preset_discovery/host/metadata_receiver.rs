use crate::preset_discovery::preset_data::Flags;
use crate::utils::{cstr_from_nullable_ptr, handle_panic};
use clack_common::utils::{Timestamp, UniversalPluginId};
use clack_host::prelude::HostError;
use clap_sys::factory::preset_discovery::clap_preset_discovery_metadata_receiver;
use clap_sys::timestamp::clap_timestamp;
use clap_sys::universal_plugin_id::clap_universal_plugin_id;
use std::ffi::{CStr, c_char};
use std::panic::AssertUnwindSafe;

/// Host-side implementation of a [metadata receiver](crate::preset_discovery::metadata_receiver).
pub trait MetadataReceiverImpl: Sized {
    /// Called when the provider wants to report to the host that an error occurred while reading metadata from a file.
    ///
    /// `error_code` is the operating system error, as returned by e.g. [`std::io::Error::raw_os_error`], if applicable.
    /// If not applicable, it should be set to a non-error value, e.g. 0 on Unix and Windows.
    fn on_error(&mut self, error_code: i32, error_message: Option<&CStr>);

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
    fn begin_preset(
        &mut self,
        name: Option<&CStr>,
        load_key: Option<&CStr>,
    ) -> Result<(), HostError>;

    /// Adds the ID of a plugin the current preset can be used with.
    fn add_plugin_id(&mut self, plugin_id: UniversalPluginId);

    /// Sets the sound pack to which the current preset belongs to.
    fn set_soundpack_id(&mut self, soundpack_id: &CStr);

    /// Sets flags specific to the current preset.
    fn set_flags(&mut self, flags: Flags);

    /// Adds a creator's name to the current preset.
    fn add_creator(&mut self, creator: &CStr);

    /// Sets the description the current preset.
    fn set_description(&mut self, description: &CStr);

    /// Sets the creation and last modification times of the current presets.
    ///
    /// If one of these is not known, it may be set to [`None`].
    fn set_timestamps(
        &mut self,
        creation_time: Option<Timestamp>,
        modification_time: Option<Timestamp>,
    );

    /// Adds a feature to the current preset.
    fn add_feature(&mut self, feature: &CStr);

    /// Adds extra metadata information to the current preset.
    fn add_extra_info(&mut self, key: &CStr, value: &CStr);
}

pub(crate) fn to_raw<M: MetadataReceiverImpl>(
    receiver: &mut M,
) -> clap_preset_discovery_metadata_receiver {
    clap_preset_discovery_metadata_receiver {
        receiver_data: (receiver as *mut M).cast(),
        on_error: Some(on_error::<M>),
        begin_preset: Some(begin_preset::<M>),
        add_plugin_id: Some(add_plugin_id::<M>),
        set_soundpack_id: Some(set_soundpack_id::<M>),
        set_flags: Some(set_flags::<M>),
        add_creator: Some(add_creator::<M>),
        set_description: Some(set_description::<M>),
        set_timestamps: Some(set_timestamps::<M>),
        add_feature: Some(add_feature::<M>),
        add_extra_info: Some(add_extra_info::<M>),
    }
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn on_error<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    error_code: i32,
    message: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let message = unsafe { cstr_from_nullable_ptr(message) };

        receiver.on_error(error_code, message);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn begin_preset<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    name: *const c_char,
    load_key: *const c_char,
) -> bool {
    handle::<M>(receiver, |receiver| {
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let name = unsafe { cstr_from_nullable_ptr(name) };
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let load_key = unsafe { cstr_from_nullable_ptr(load_key) };

        receiver
            .begin_preset(name, load_key)
            .map_err(|_| ReceiverError)
    })
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn add_plugin_id<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    plugin_id: *const clap_universal_plugin_id,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let plugin_id =
            unsafe { UniversalPluginId::from_raw_ptr(plugin_id) }.ok_or(ReceiverError)?;

        receiver.add_plugin_id(plugin_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn set_soundpack_id<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    soundpack_id: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let soundpack_id = unsafe { cstr_from_nullable_ptr(soundpack_id) }.ok_or(ReceiverError)?;

        receiver.set_soundpack_id(soundpack_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn set_flags<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    flags: u32,
) {
    handle::<M>(receiver, |receiver| {
        let flags = Flags::from_bits_truncate(flags);

        receiver.set_flags(flags);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn add_creator<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    soundpack_id: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let soundpack_id = unsafe { cstr_from_nullable_ptr(soundpack_id) }.ok_or(ReceiverError)?;

        receiver.add_creator(soundpack_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn set_description<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    soundpack_id: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let soundpack_id = unsafe { cstr_from_nullable_ptr(soundpack_id) }.ok_or(ReceiverError)?;

        receiver.set_description(soundpack_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn set_timestamps<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    creation_time: clap_timestamp,
    modification_time: clap_timestamp,
) {
    handle::<M>(receiver, |receiver| {
        let creation_time = Timestamp::from_raw(creation_time);
        let modification_time = Timestamp::from_raw(modification_time);

        receiver.set_timestamps(creation_time, modification_time);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn add_feature<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    soundpack_id: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let soundpack_id = unsafe { cstr_from_nullable_ptr(soundpack_id) }.ok_or(ReceiverError)?;

        receiver.add_feature(soundpack_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn add_extra_info<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    key: *const c_char,
    value: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let key = unsafe { cstr_from_nullable_ptr(key) }.ok_or(ReceiverError)?;
        // SAFETY: String pointer is guaranteed to be valid by the CLAP spec
        let value = unsafe { cstr_from_nullable_ptr(value) }.ok_or(ReceiverError)?;

        receiver.add_extra_info(key, value);

        Ok(())
    });
}

/// # Safety
///
/// `receiver` must be valid and come from `to_raw().receiver_data`.
#[inline]
unsafe fn handle<M: MetadataReceiverImpl>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    handler: impl FnOnce(&mut M) -> Result<(), ReceiverError>,
) -> bool {
    if receiver.is_null() {
        return false;
    };

    // SAFETY: CLAP spec guarantees this is valid for reads
    let receiver = unsafe { receiver.read() };
    let receiver = receiver.receiver_data.cast::<M>();

    // SAFETY: We created that pointer ourselves from an exclusive &mut reference.
    // The clap spec enforces no two M methods can be called simultaneously, so this reference is unique.
    let receiver = unsafe { receiver.as_mut() };

    let Some(receiver) = receiver else {
        return false;
    };

    let result = handle_panic(AssertUnwindSafe(|| handler(receiver)));

    match result {
        Ok(Err(_)) => false,
        Err(_) => false,
        Ok(Ok(())) => true,
    }
}

struct ReceiverError;
