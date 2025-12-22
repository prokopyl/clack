use crate::preset_discovery::Flags;
use crate::utils::{cstr_from_nullable_ptr, handle_panic};
use clack_common::utils::{Timestamp, UniversalPluginID};
use clap_sys::factory::preset_discovery::clap_preset_discovery_metadata_receiver;
use clap_sys::timestamp::clap_timestamp;
use clap_sys::universal_plugin_id::clap_universal_plugin_id;
use std::ffi::{CStr, c_char};
use std::panic::AssertUnwindSafe;

pub trait MetadataReceiver: Sized {
    fn on_error(&mut self, error_code: i32, error_message: Option<&CStr>);
    // TODO: handle errors?
    fn begin_preset(&mut self, name: Option<&CStr>, load_key: Option<&CStr>);
    fn add_plugin_id(&mut self, plugin_id: UniversalPluginID);
    fn set_soundpack_id(&mut self, soundpack_id: &CStr);
    fn set_flags(&mut self, flags: Flags);
    fn add_creator(&mut self, creator: &CStr);
    fn set_description(&mut self, description: &CStr);
    fn set_timestamps(
        &mut self,
        creation_time: Option<Timestamp>,
        modification_time: Option<Timestamp>,
    );
    fn add_feature(&mut self, feature: &CStr);
    fn add_extra_info(&mut self, key: &CStr, value: &CStr);
}

pub(crate) fn to_raw<M: MetadataReceiver>(
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
unsafe extern "C" fn on_error<M: MetadataReceiver>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    error_code: i32,
    message: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: TODO
        let message = unsafe { cstr_from_nullable_ptr(message) };

        receiver.on_error(error_code, message);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn begin_preset<M: MetadataReceiver>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    name: *const c_char,
    load_key: *const c_char,
) -> bool {
    handle::<M>(receiver, |receiver| {
        // SAFETY: TODO
        let name = unsafe { cstr_from_nullable_ptr(name) };
        // SAFETY: TODO
        let load_key = unsafe { cstr_from_nullable_ptr(load_key) };

        receiver.begin_preset(name, load_key);

        Ok(())
    })
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn add_plugin_id<M: MetadataReceiver>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    plugin_id: *const clap_universal_plugin_id,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: TODO
        let plugin_id =
            unsafe { UniversalPluginID::from_raw_ptr(plugin_id) }.ok_or(ReceiverError)?;

        receiver.add_plugin_id(plugin_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn set_soundpack_id<M: MetadataReceiver>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    soundpack_id: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: TODO
        let soundpack_id = unsafe { cstr_from_nullable_ptr(soundpack_id) }.ok_or(ReceiverError)?;

        receiver.set_soundpack_id(soundpack_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn set_flags<M: MetadataReceiver>(
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
unsafe extern "C" fn add_creator<M: MetadataReceiver>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    soundpack_id: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: TODO
        let soundpack_id = unsafe { cstr_from_nullable_ptr(soundpack_id) }.ok_or(ReceiverError)?;

        receiver.add_creator(soundpack_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn set_description<M: MetadataReceiver>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    soundpack_id: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: TODO
        let soundpack_id = unsafe { cstr_from_nullable_ptr(soundpack_id) }.ok_or(ReceiverError)?;

        receiver.set_description(soundpack_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn set_timestamps<M: MetadataReceiver>(
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
unsafe extern "C" fn add_feature<M: MetadataReceiver>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    soundpack_id: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: TODO
        let soundpack_id = unsafe { cstr_from_nullable_ptr(soundpack_id) }.ok_or(ReceiverError)?;

        receiver.add_feature(soundpack_id);

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn add_extra_info<M: MetadataReceiver>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    key: *const c_char,
    value: *const c_char,
) {
    handle::<M>(receiver, |receiver| {
        // SAFETY: TODO
        let key = unsafe { cstr_from_nullable_ptr(key) }.ok_or(ReceiverError)?;
        // SAFETY: TODO
        let value = unsafe { cstr_from_nullable_ptr(value) }.ok_or(ReceiverError)?;

        receiver.add_extra_info(key, value);

        Ok(())
    });
}

#[inline]
unsafe fn handle<M: MetadataReceiver>(
    receiver: *const clap_preset_discovery_metadata_receiver,
    handler: impl FnOnce(&mut M) -> Result<(), ReceiverError>,
) -> bool {
    if receiver.is_null() {
        return false;
    };

    // SAFETY: TODO
    let receiver = unsafe { receiver.read() };
    let receiver = receiver.receiver_data.cast::<M>();

    // SAFETY: TODO
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
