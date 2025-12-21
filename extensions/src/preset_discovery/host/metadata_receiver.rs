use crate::preset_discovery::Flags;
use clack_common::utils::Timestamp;
use clap_sys::factory::preset_discovery::clap_preset_discovery_metadata_receiver;
use std::ffi::CStr;
use std::marker::PhantomData;

pub trait MetadataReceiver {
    fn on_error(&mut self, error_code: i32, error_message: Option<&CStr>);
    // TODO: handle errors
    fn begin_preset(&mut self, name: Option<&CStr>, load_key: Option<&CStr>);
    // TODO: universal plugin ID
    fn add_plugin_id(&mut self, plugin_id: &CStr);
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

pub(crate) struct RawMetadataReceiver<'a> {
    inner: clap_preset_discovery_metadata_receiver,
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a> RawMetadataReceiver<'a> {
    pub fn from_impl<M: MetadataReceiver>(receiver: &'a mut M) {
        todo!()
    }

    pub fn as_raw(&mut self) -> *mut clap_preset_discovery_metadata_receiver {
        &mut self.inner
    }
}
