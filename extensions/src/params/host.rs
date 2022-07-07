use super::*;
use crate::params::info::ParamInfo;
use clack_common::events::io::{InputEvents, OutputEvents};
use clack_common::extensions::ExtensionImplementation;
use clack_host::host::PluginHoster;
use clack_host::instance::processor::StoppedPluginAudioProcessor;
use clack_host::instance::PluginInstance;
use clack_host::wrapper::HostWrapper;
use clap_sys::host::clap_host;
use std::ffi::CStr;
use std::mem::MaybeUninit;

impl PluginParams {
    pub fn count<H: for<'a> PluginHoster<'a>>(&self, plugin: &PluginInstance<H>) -> u32 {
        unsafe { (self.0.count)(plugin.raw_instance()) }
    }

    pub fn get_info<'b, H: for<'a> PluginHoster<'a>>(
        &self,
        plugin: &PluginInstance<H>,
        param_index: u32,
        info: &'b mut MaybeUninit<ParamInfo>,
    ) -> Option<&'b mut ParamInfo> {
        let valid = unsafe {
            (self.0.get_info)(
                plugin.raw_instance(),
                param_index,
                info.as_mut_ptr() as *mut _,
            )
        };

        if valid {
            unsafe { Some(info.assume_init_mut()) }
        } else {
            None
        }
    }

    pub fn get_value<H: for<'a> PluginHoster<'a>>(
        &self,
        plugin: &PluginInstance<H>,
        param_id: u32,
    ) -> Option<f64> {
        let mut value = MaybeUninit::uninit();
        let valid =
            unsafe { (self.0.get_value)(plugin.raw_instance(), param_id, value.as_mut_ptr()) };

        if valid {
            unsafe { Some(value.assume_init()) }
        } else {
            None
        }
    }

    pub fn value_to_text<'b, H: for<'a> PluginHoster<'a>>(
        &self,
        plugin: &PluginInstance<H>,
        param_id: u32,
        value: f64,
        buffer: &'b mut [MaybeUninit<u8>],
    ) -> Option<&'b mut [u8]> {
        let valid = unsafe {
            (self.0.value_to_text)(
                plugin.raw_instance(),
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

    pub fn text_to_value<H: for<'a> PluginHoster<'a>>(
        &self,
        plugin: &PluginInstance<H>,
        param_id: u32,
        display: &CStr,
    ) -> Option<f64> {
        let mut value = MaybeUninit::uninit();

        let valid = unsafe {
            (self.0.text_to_value)(
                plugin.raw_instance(),
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
    pub fn flush_inactive<H: for<'a> PluginHoster<'a>>(
        &self,
        plugin: &mut PluginInstance<H>,
        input_event_list: &InputEvents,
        output_event_list: &mut OutputEvents,
    ) -> bool {
        if plugin.is_active() {
            return false;
        }

        unsafe {
            (self.0.flush)(
                plugin.raw_instance(),
                input_event_list.as_raw(),
                output_event_list.as_raw_mut(),
            )
        };
        true
    }

    pub fn flush_active<H: for<'a> PluginHoster<'a>>(
        &self,
        plugin: &mut StoppedPluginAudioProcessor<H>, // TODO: separate handle type
        input_event_list: &InputEvents,
        output_event_list: &mut OutputEvents,
    ) {
        // SAFETY: flush is already guaranteed by the types to be called on an active, non-processing plugin
        unsafe {
            (self.0.flush)(
                plugin.audio_processor_plugin_data().as_raw(),
                input_event_list.as_raw(),
                output_event_list.as_raw_mut(),
            )
        }
    }
}

#[inline]
unsafe fn assume_init_slice<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}

pub trait HostParamsImplementation {
    fn request_flush(&self);
}

pub trait HostParamsImplementationMainThread {
    fn rescan(&mut self, flags: ParamRescanFlags);
    fn clear(&mut self, param_id: u32, flags: ParamClearFlags);
}

impl<H: for<'a> PluginHoster<'a>> ExtensionImplementation<H> for HostParams
where
    for<'a> <H as PluginHoster<'a>>::Shared: HostParamsImplementation,
    for<'a> <H as PluginHoster<'a>>::MainThread: HostParamsImplementationMainThread,
{
    const IMPLEMENTATION: &'static Self = &HostParams(clap_host_params {
        rescan: rescan::<H>,
        clear: clear::<H>,
        request_flush: request_flush::<H>,
    });
}

unsafe extern "C" fn rescan<H: for<'a> PluginHoster<'a>>(
    host: *const clap_host,
    flags: clap_param_rescan_flags,
) where
    for<'a> <H as PluginHoster<'a>>::MainThread: HostParamsImplementationMainThread,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(ParamRescanFlags::from_bits_truncate(flags));

        Ok(())
    });
}

unsafe extern "C" fn clear<H: for<'a> PluginHoster<'a>>(
    host: *const clap_host,
    param_id: u32,
    flags: clap_param_clear_flags,
) where
    for<'a> <H as PluginHoster<'a>>::MainThread: HostParamsImplementationMainThread,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .clear(param_id, ParamClearFlags::from_bits_truncate(flags));

        Ok(())
    });
}

unsafe extern "C" fn request_flush<H: for<'a> PluginHoster<'a>>(host: *const clap_host)
where
    for<'a> <H as PluginHoster<'a>>::Shared: HostParamsImplementation,
{
    HostWrapper::<H>::handle(host, |host| {
        host.shared().request_flush();

        Ok(())
    });
}
