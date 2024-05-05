use super::*;
use clack_common::events::io::{InputEvents, OutputEvents};
use clack_host::extensions::prelude::*;
use std::mem::MaybeUninit;

#[derive(Clone)]
pub struct ParamInfoBuffer {
    inner: MaybeUninit<clap_param_info>,
}

impl Default for ParamInfoBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl ParamInfoBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
        }
    }
}

impl PluginParams {
    pub fn count(&self, plugin: &mut PluginMainThreadHandle) -> u32 {
        match plugin.use_extension(&self.0).count {
            None => 0,
            // SAFETY: This type ensures the function pointer is valid.
            Some(count) => unsafe { count(plugin.as_raw()) },
        }
    }

    pub fn get_info<'b>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        index: u32,
        buffer: &'b mut ParamInfoBuffer,
    ) -> Option<ParamInfo<'b>> {
        // SAFETY: This type ensures the function pointer is valid.
        let success = unsafe {
            plugin.use_extension(&self.0).get_info?(
                plugin.as_raw(),
                index,
                buffer.inner.as_mut_ptr(),
            )
        };

        if success {
            // SAFETY: we just checked the buffer was successfully written to.
            unsafe { ParamInfo::from_raw(buffer.inner.assume_init_mut()) }
        } else {
            None
        }
    }

    pub fn get_value(&self, plugin: &mut PluginMainThreadHandle, param_id: u32) -> Option<f64> {
        let mut value = MaybeUninit::uninit();
        // SAFETY: This type ensures the function pointer is valid.
        let valid = unsafe {
            plugin.use_extension(&self.0).get_value?(plugin.as_raw(), param_id, value.as_mut_ptr())
        };

        if valid {
            // SAFETY: we just checked the value was successfully written to.
            unsafe { Some(value.assume_init()) }
        } else {
            None
        }
    }

    pub fn value_to_text<'b>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        param_id: u32,
        value: f64,
        buffer: &'b mut [MaybeUninit<u8>],
    ) -> Result<&'b mut [u8], core::fmt::Error> {
        let Some(value_to_text) = plugin.use_extension(&self.0).value_to_text else {
            return Err(core::fmt::Error);
        };

        // SAFETY: This type ensures the function pointer is valid.
        let valid = unsafe {
            value_to_text(
                plugin.as_raw(),
                param_id,
                value,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
            )
        };

        if !valid {
            return Err(core::fmt::Error);
        }

        // SAFETY: technically the whole buffer may not be fully initialized, but uninit u8 is fine
        let buffer = unsafe { assume_init_slice(buffer) };
        // If no nul byte found, we take the entire buffer
        let buffer_total_len = buffer.iter().position(|b| *b == 0).unwrap_or(buffer.len());
        Ok(&mut buffer[..buffer_total_len])
    }

    pub fn text_to_value(
        &self,
        plugin: &mut PluginMainThreadHandle,
        param_id: u32,
        display: &CStr,
    ) -> Option<f64> {
        let mut value = MaybeUninit::uninit();

        // SAFETY: This type ensures the function pointer is valid.
        let valid = unsafe {
            plugin.use_extension(&self.0).text_to_value?(
                plugin.as_raw(),
                param_id,
                display.as_ptr(),
                value.as_mut_ptr(),
            )
        };

        if valid {
            // SAFETY: We just checked the buffer was successfully written to.
            unsafe { Some(value.assume_init()) }
        } else {
            None
        }
    }

    // TODO: find a way to enforce !active | !processing ?
    pub fn flush(
        &self,
        plugin: &mut PluginMainThreadHandle,
        input_parameter_changes: &InputEvents,
        output_parameter_changes: &mut OutputEvents,
    ) {
        if let Some(flush) = plugin.use_extension(&self.0).flush {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe {
                flush(
                    plugin.as_raw(),
                    input_parameter_changes.as_raw(),
                    output_parameter_changes.as_raw_mut(),
                )
            }
        }
    }

    // TODO: find a way to enforce !active | !processing ?
    pub fn flush_active(
        &self,
        plugin: &mut PluginAudioProcessorHandle,
        input_parameter_changes: &InputEvents,
        output_parameter_changes: &mut OutputEvents,
    ) {
        if let Some(flush) = plugin.use_extension(&self.0).flush {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe {
                flush(
                    plugin.as_raw(),
                    input_parameter_changes.as_raw(),
                    output_parameter_changes.as_raw_mut(),
                )
            }
        }
    }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
unsafe fn assume_init_slice<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}

pub trait HostParamsImplShared {
    fn request_flush(&self);
}

pub trait HostParamsImplMainThread {
    fn rescan(&mut self, flags: ParamRescanFlags);
    fn clear(&mut self, param_id: u32, flags: ParamClearFlags);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H: HostHandlers> ExtensionImplementation<H> for HostParams
where
    for<'a> <H as HostHandlers>::Shared<'a>: HostParamsImplShared,
    for<'a> <H as HostHandlers>::MainThread<'a>: HostParamsImplMainThread,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_params {
            rescan: Some(rescan::<H>),
            clear: Some(clear::<H>),
            request_flush: Some(request_flush::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn rescan<H: HostHandlers>(host: *const clap_host, flags: clap_param_rescan_flags)
where
    for<'a> <H as HostHandlers>::MainThread<'a>: HostParamsImplMainThread,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(ParamRescanFlags::from_bits_truncate(flags));

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn clear<H: HostHandlers>(
    host: *const clap_host,
    param_id: u32,
    flags: clap_param_clear_flags,
) where
    for<'a> <H as HostHandlers>::MainThread<'a>: HostParamsImplMainThread,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .clear(param_id, ParamClearFlags::from_bits_truncate(flags));

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn request_flush<H: HostHandlers>(host: *const clap_host)
where
    for<'a> <H as HostHandlers>::Shared<'a>: HostParamsImplShared,
{
    HostWrapper::<H>::handle(host, |host| {
        host.shared().request_flush();

        Ok(())
    });
}
