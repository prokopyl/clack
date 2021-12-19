use clap_plugin::extension::ExtensionDescriptor;
use clap_plugin::plugin::{Plugin, PluginInstance};
use clap_sys::events::clap_event_list;
use clap_sys::ext::params::{clap_param_info, clap_plugin_params, CLAP_EXT_PARAMS};
use clap_sys::id::clap_id;
use std::ffi::CStr;
use std::marker::PhantomData;

pub struct ParamsDescriptor<P>(PhantomData<P>);

pub struct ParamInfo;

pub trait PluginParams<'a>: Plugin<'a> {
    fn count(&self) -> u32;
    fn get_info(&self, param_index: i32, info: &mut ParamInfo) -> bool; // TODO: meh
    fn get_value(&self, param_id: u32) -> Option<f64>;
    fn value_to_text(&self, param_id: u32, value: f64, output_buf: &mut [u8]) -> bool; // TODO: meh
    fn text_to_value(&self, param_id: u32, text: &str) -> Option<f64>;
    fn flush(&self); // TODO: implement this
}

extern "C" fn count<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
) -> u32 {
    unsafe { P::count(PluginInstance::get_plugin(plugin)) }
}

extern "C" fn get_info<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_index: i32,
    value: *mut clap_param_info,
) -> bool {
    let mut info = ParamInfo; // TODO
    unsafe { P::get_info(PluginInstance::get_plugin(plugin), param_index, &mut info) }
}

extern "C" fn get_value<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_id: clap_id,
    value: *mut f64,
) -> bool {
    let val = unsafe { P::get_value(PluginInstance::get_plugin(plugin), param_id) };
    match val {
        None => false,
        Some(val) => {
            unsafe {
                *value = val;
            }
            true
        }
    }
}

extern "C" fn value_to_text<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_id: clap_id,
    value: f64,
    display: *mut ::std::os::raw::c_char,
    size: u32,
) -> bool {
    let buf = unsafe { ::core::slice::from_raw_parts_mut(display as *mut u8, size as usize) };
    unsafe { P::value_to_text(PluginInstance::get_plugin(plugin), param_id, value, buf) }
}

extern "C" fn text_to_value<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    param_id: clap_id,
    display: *const ::std::os::raw::c_char,
    value: *mut f64,
) -> bool {
    let display = unsafe { CStr::from_ptr(display) }.to_bytes();
    let display = ::core::str::from_utf8(display).unwrap(); // TODO: unsafe unwrap
    let val = unsafe { P::text_to_value(PluginInstance::get_plugin(plugin), param_id, display) };
    match val {
        None => false,
        Some(val) => {
            unsafe {
                *value = val;
            }
            true
        }
    }
}

extern "C" fn flush<'a, P: PluginParams<'a>>(
    plugin: *const ::clap_sys::plugin::clap_plugin,
    _input_parameter_changes: *const clap_event_list,
    _output_event_list: *const clap_event_list,
) {
    unsafe { P::flush(PluginInstance::get_plugin(plugin)) } // TODO
}

impl<'a, P: PluginParams<'a>> ExtensionDescriptor<P> for ParamsDescriptor<P> {
    const IDENTIFIER: *const u8 = CLAP_EXT_PARAMS as *const _;
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
