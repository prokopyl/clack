use crate::params::info::ParamInfoData;
use crate::utils::write_to_array_buf;
use clack_common::events::io::{InputEvents, OutputEvents};
use clack_common::extensions::ExtensionImplementation;
use clack_plugin::plugin::wrapper::{PluginWrapper, PluginWrapperError};
use clack_plugin::plugin::Plugin;
use clap_sys::events::{clap_input_events, clap_output_events};
use clap_sys::ext::log::CLAP_LOG_ERROR;
use clap_sys::ext::params::{clap_param_info, clap_plugin_params};
use clap_sys::id::clap_id;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

pub struct ParamInfoWriter<'a> {
    initialized: bool,
    inner: &'a mut MaybeUninit<clap_param_info>,
}

impl<'a> ParamInfoWriter<'a> {
    fn new(ptr: *mut clap_param_info) -> Self {
        Self {
            initialized: false,
            // SAFETY: MaybeUninit<T> and T have same memory representation
            inner: unsafe { &mut *(ptr as *mut _) },
        }
    }

    #[inline]
    pub fn set(&mut self, param: &ParamInfoData) {
        let buf = self.inner.as_mut_ptr();

        unsafe {
            (core::ptr::addr_of_mut!((*buf).id) as *mut u32).write(param.id);
            (core::ptr::addr_of_mut!((*buf).flags) as *mut u32).write(param.flags.bits());
            (core::ptr::addr_of_mut!((*buf).min_value) as *mut f64).write(param.min_value);
            (core::ptr::addr_of_mut!((*buf).max_value) as *mut f64).write(param.max_value);
            (core::ptr::addr_of_mut!((*buf).default_value) as *mut f64).write(param.default_value);
            (core::ptr::addr_of_mut!((*buf).cookie) as *mut *mut c_void).write(param.cookie);

            write_to_array_buf(core::ptr::addr_of_mut!((*buf).name), &param.name);
            write_to_array_buf(core::ptr::addr_of_mut!((*buf).module), &param.module);
        }
        self.initialized = true;
    }
}

pub struct ParamDisplayWriter<'a> {
    cursor_position: usize,
    buffer: &'a mut [u8],
}

impl<'a> ParamDisplayWriter<'a> {
    #[inline]
    fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            cursor_position: 0,
            buffer,
        }
    }

    #[inline]
    #[allow(clippy::len_without_is_empty)] // Len should never be 0, unless host is misbehaving
    pub fn len(&self) -> usize {
        self.buffer.len().saturating_sub(1)
    }

    #[inline]
    pub fn remaining_len(&self) -> usize {
        self.buffer.len().saturating_sub(self.cursor_position + 1)
    }

    fn finish(self) -> bool {
        if self.cursor_position > 0 {
            self.buffer[self.cursor_position] = 0;
        }
        self.cursor_position > 0
    }
}

impl<'a> ::core::fmt::Write for ParamDisplayWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s = s.as_bytes();
        if s.len() > self.remaining_len() {
            return Err(::core::fmt::Error);
        }

        self.buffer[self.cursor_position..self.cursor_position + s.len()].copy_from_slice(s);
        self.cursor_position += s.len();

        Ok(())
    }
}

pub trait PluginMainThreadParams<'a> {
    fn count(&self) -> u32;
    fn get_info(&self, param_index: u32, info: &mut ParamInfoWriter);
    fn get_value(&self, param_id: u32) -> Option<f64>;
    fn value_to_text(
        &self,
        param_id: u32,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> ::core::fmt::Result;
    fn text_to_value(&self, param_id: u32, text: &str) -> Option<f64>;
    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        output_parameter_changes: &mut OutputEvents,
    );
}

pub trait PluginParamsImpl<'a>: Plugin<'a>
where
    Self::MainThread: PluginMainThreadParams<'a>,
{
    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        output_parameter_changes: &mut OutputEvents,
    );
}

unsafe extern "C" fn count<'a, P: PluginParamsImpl<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
) -> u32
where
    P::MainThread: PluginMainThreadParams<'a>,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        Ok(P::MainThread::count(p.main_thread().as_ref()))
    })
    .unwrap_or(0)
}

unsafe extern "C" fn get_info<'a, P: PluginParamsImpl<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_index: u32,
    value: *mut clap_param_info,
) -> bool
where
    P::MainThread: PluginMainThreadParams<'a>,
{
    let mut info = ParamInfoWriter::new(value);
    PluginWrapper::<P>::handle(plugin, |p| {
        P::MainThread::get_info(p.main_thread().as_ref(), param_index, &mut info);
        Ok(())
    })
    .is_some()
        && info.initialized
}

unsafe extern "C" fn get_value<'a, P: PluginParamsImpl<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_id: clap_id,
    value: *mut f64,
) -> bool
where
    P::MainThread: PluginMainThreadParams<'a>,
{
    let val = PluginWrapper::<P>::handle(plugin, |p| {
        Ok(P::MainThread::get_value(p.main_thread().as_ref(), param_id))
    })
    .flatten();

    match val {
        None => false,
        Some(val) => {
            *value = val;
            true
        }
    }
}

unsafe extern "C" fn value_to_text<'a, P: PluginParamsImpl<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_id: clap_id,
    value: f64,
    display: *mut ::std::os::raw::c_char,
    size: u32,
) -> bool
where
    P::MainThread: PluginMainThreadParams<'a>,
{
    let buf = ::core::slice::from_raw_parts_mut(display as *mut u8, size as usize);
    let mut writer = ParamDisplayWriter::new(buf);
    PluginWrapper::<P>::handle(plugin, |p| {
        P::MainThread::value_to_text(p.main_thread().as_ref(), param_id, value, &mut writer)
            .map_err(PluginWrapperError::with_severity(CLAP_LOG_ERROR as i32))
    })
    .is_some()
        && writer.finish()
}

unsafe extern "C" fn text_to_value<'a, P: PluginParamsImpl<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_id: clap_id,
    display: *const ::std::os::raw::c_char,
    value: *mut f64,
) -> bool
where
    P::MainThread: PluginMainThreadParams<'a>,
{
    let display = CStr::from_ptr(display).to_bytes();

    let val = PluginWrapper::<P>::handle(plugin, |p| {
        let display = ::core::str::from_utf8(display)
            .map_err(PluginWrapperError::with_severity(CLAP_LOG_ERROR as i32))?;
        Ok(P::MainThread::text_to_value(
            p.main_thread().as_ref(),
            param_id,
            display,
        ))
    })
    .flatten();

    match val {
        None => false,
        Some(val) => {
            *value = val;
            true
        }
    }
}

unsafe extern "C" fn flush<'a, P: PluginParamsImpl<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    input_parameter_changes: *const clap_input_events,
    output_parameter_changes: *const clap_output_events,
) where
    P::MainThread: PluginMainThreadParams<'a>,
{
    let input_parameter_changes = InputEvents::from_raw(&*input_parameter_changes);
    let output_parameter_changes =
        OutputEvents::from_raw_mut(&mut *(output_parameter_changes as *mut _));

    PluginWrapper::<P>::handle(plugin, |p| {
        if let Ok(mut audio) = p.audio_processor() {
            P::flush(
                audio.as_mut(),
                input_parameter_changes,
                output_parameter_changes,
            );
        } else {
            P::MainThread::flush(
                p.main_thread().as_mut(),
                input_parameter_changes,
                output_parameter_changes,
            );
        }
        Ok(())
    });
}

impl<'a, P: PluginParamsImpl<'a>> ExtensionImplementation<P> for super::PluginParams
where
    P::MainThread: PluginMainThreadParams<'a>,
{
    const IMPLEMENTATION: &'static Self = &super::PluginParams(
        clap_plugin_params {
            count: Some(count::<P>),
            get_info: Some(get_info::<P>),
            get_value: Some(get_value::<P>),
            value_to_text: Some(value_to_text::<P>),
            text_to_value: Some(text_to_value::<P>),
            flush: Some(flush::<P>),
        },
        PhantomData,
    );
}
