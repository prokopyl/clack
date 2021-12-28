use clap_audio_common::events::list::EventList;
use clap_audio_common::extensions::{Extension, ExtensionDescriptor};
use clap_audio_plugin::plugin::wrapper::{handle_plugin, handle_plugin_returning};
use clap_audio_plugin::plugin::{Plugin, PluginError};
use clap_sys::events::clap_event_list;
use clap_sys::ext::params::{clap_param_info, clap_plugin_params, CLAP_EXT_PARAMS};
use clap_sys::id::clap_id;
use std::ffi::{c_void, CStr};
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::str::Utf8Error;

// TODO
pub struct ParamsDescriptor(clap_plugin_params);

pub struct ParamInfo {
    inner: clap_param_info,
}

impl ParamInfo {}

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
    pub fn set(&mut self, param: &ParamInfo) {
        self.inner.write(param.inner);
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

        Ok(())
    }
}

pub trait PluginParams<'a>: Plugin<'a> {
    fn count(&self) -> u32;
    fn get_info(&self, param_index: i32, info: &mut ParamInfoWriter);
    fn get_value(&self, param_id: u32) -> Option<f64>;
    fn value_to_text(
        &self,
        param_id: u32,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> ::core::fmt::Result;
    fn text_to_value(&self, param_id: u32, text: &str) -> Option<f64>;
    fn flush(&self, input_parameter_changes: &EventList, output_parameter_changes: &EventList);
}

unsafe extern "C" fn count<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
) -> u32 {
    handle_plugin_returning::<_, _, _, PluginError>(plugin, |p| Ok(P::count(p))).unwrap_or(0)
}

unsafe extern "C" fn get_info<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_index: i32,
    value: *mut clap_param_info,
) -> bool {
    let mut info = ParamInfoWriter::new(value);
    handle_plugin::<_, _, PluginError>(plugin, |p| {
        P::get_info(p, param_index, &mut info);
        Ok(())
    }) && info.initialized
}

unsafe extern "C" fn get_value<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_id: clap_id,
    value: *mut f64,
) -> bool {
    let val =
        handle_plugin_returning::<_, _, _, PluginError>(plugin, |p| Ok(P::get_value(p, param_id)))
            .flatten();

    match val {
        None => false,
        Some(val) => {
            *value = val;
            true
        }
    }
}

unsafe extern "C" fn value_to_text<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_id: clap_id,
    value: f64,
    display: *mut ::std::os::raw::c_char,
    size: u32,
) -> bool {
    let buf = ::core::slice::from_raw_parts_mut(display as *mut u8, size as usize);
    let mut writer = ParamDisplayWriter::new(buf);
    handle_plugin::<_, _, _>(plugin, |p| {
        P::value_to_text(p, param_id, value, &mut writer)
    }) && writer.finish()
}

unsafe extern "C" fn text_to_value<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_id: clap_id,
    display: *const ::std::os::raw::c_char,
    value: *mut f64,
) -> bool {
    let display = CStr::from_ptr(display).to_bytes();

    let val = handle_plugin_returning::<_, _, _, Utf8Error>(plugin, |p| {
        let display = ::core::str::from_utf8(display)?;
        Ok(P::text_to_value(p, param_id, display))
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

unsafe extern "C" fn flush<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    input_parameter_changes: *const clap_event_list,
    output_parameter_changes: *const clap_event_list,
) {
    let input_parameter_changes = EventList::from_raw(input_parameter_changes);
    let output_parameter_changes = EventList::from_raw(output_parameter_changes);

    handle_plugin::<_, _, PluginError>(plugin, |p| {
        P::flush(p, input_parameter_changes, output_parameter_changes);
        Ok(())
    });
}

unsafe impl<'a> Extension<'a> for ParamsDescriptor {
    const IDENTIFIER: *const u8 = CLAP_EXT_PARAMS as *const _;

    // TODO: this may be redundant
    unsafe fn from_extension_ptr(ptr: NonNull<c_void>) -> &'a Self {
        ptr.cast().as_ref()
    }
}

unsafe impl<'a, P: PluginParams<'a>> ExtensionDescriptor<'a, P> for ParamsDescriptor {
    type ExtensionInterface = clap_plugin_params;
    const INTERFACE: &'static Self::ExtensionInterface = &clap_plugin_params {
        count: count::<P>,
        get_info: get_info::<P>,
        get_value: get_value::<P>,
        value_to_text: value_to_text::<P>,
        text_to_value: text_to_value::<P>,
        flush: flush::<P>,
    };
}
