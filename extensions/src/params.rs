use crate::params::info::ParamInfo;
use clack_common::events::list::EventList;
use clack_common::extensions::Extension;
use clack_host::instance::channel::PluginInstanceChannelSend;
use clack_host::instance::processor::StoppedPluginAudioProcessor;
use clack_host::instance::PluginInstance;
use clap_sys::ext::params::{clap_plugin_params, CLAP_EXT_PARAMS};
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

#[repr(C)]
pub struct PluginParams(clap_plugin_params, PhantomData<*const clap_plugin_params>);

pub mod implementation;
pub mod info;

unsafe impl<'a> Extension<'a> for PluginParams {
    const IDENTIFIER: *const u8 = CLAP_EXT_PARAMS as *const _;

    // TODO: this may be redundant
    #[inline]
    unsafe fn from_extension_ptr(ptr: NonNull<c_void>) -> &'a Self {
        ptr.cast().as_ref()
    }
}

impl PluginParams {
    pub fn count(&self, plugin: &PluginInstance) -> u32 {
        unsafe { (self.0.count)(plugin.as_raw()) }
    }

    pub fn get_info<'b>(
        &self,
        plugin: &PluginInstance,
        param_index: i32,
        info: &'b mut MaybeUninit<ParamInfo>,
    ) -> Option<&'b mut ParamInfo> {
        let valid =
            unsafe { (self.0.get_info)(plugin.as_raw(), param_index, info.as_mut_ptr() as *mut _) };

        if valid {
            unsafe { Some(info.assume_init_mut()) }
        } else {
            None
        }
    }

    pub fn get_value(&self, plugin: &PluginInstance, param_id: u32) -> Option<f64> {
        let mut value = MaybeUninit::uninit();
        let valid = unsafe { (self.0.get_value)(plugin.as_raw(), param_id, value.as_mut_ptr()) };

        if valid {
            unsafe { Some(value.assume_init()) }
        } else {
            None
        }
    }

    pub fn value_to_text<'b>(
        &self,
        plugin: &PluginInstance,
        param_id: u32,
        value: f64,
        buffer: &'b mut [std::mem::MaybeUninit<u8>],
    ) -> Option<&'b mut [u8]> {
        let valid = unsafe {
            (self.0.value_to_text)(
                plugin.as_raw(),
                param_id,
                value,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
            )
        };

        if valid {
            // SAFETY: technically not all of the buffer may be initialized, but uninit u8 is fine
            let buffer = unsafe { assume_init_slice(buffer) };
            // If no nul byte found, we take the entire buffer
            let buffer_total_len = buffer.iter().position(|b| *b == 0).unwrap_or(buffer.len());
            Some(&mut buffer[..buffer_total_len])
        } else {
            None
        }
    }

    pub fn text_to_value(
        &self,
        plugin: &PluginInstance,
        param_id: u32,
        display: &CStr,
    ) -> Option<f64> {
        let mut value = MaybeUninit::uninit();
        let valid = unsafe {
            (self.0.text_to_value)(
                plugin.as_raw(),
                param_id,
                display.as_ptr(),
                value.as_mut_ptr(),
            )
        };

        if valid {
            unsafe { Some(value.assume_init()) }
        } else {
            None
        }
    }

    // TODO: return a proper error
    pub fn flush_inactive(
        &self,
        plugin: &mut PluginInstance,
        input_event_list: &mut EventList,
        output_event_list: &mut EventList,
    ) -> bool {
        if plugin.is_active() {
            return false;
        }

        unsafe {
            (self.0.flush)(
                plugin.as_raw(),
                input_event_list.as_raw_mut(),
                output_event_list.as_raw_mut(),
            )
        };

        true
    }

    pub fn flush_active<TChannel: PluginInstanceChannelSend>(
        &self,
        plugin: &mut StoppedPluginAudioProcessor<TChannel>,
        input_event_list: &mut EventList,
        output_event_list: &mut EventList,
    ) {
        // SAFETY: flush is already guaranteed by the types to be called on an active, non-processing plugin
        unsafe {
            (self.0.flush)(
                plugin.as_raw(),
                input_event_list.as_raw_mut(),
                output_event_list.as_raw_mut(),
            )
        };
    }
}

#[inline]
unsafe fn assume_init_slice<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}
