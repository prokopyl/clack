use super::*;
use crate::utils::{slice_from_external_parts_mut, write_to_array_buf};
use clack_common::events::io::{InputEvents, OutputEvents};
use clack_plugin::extensions::prelude::*;
use clap_sys::events::{clap_input_events, clap_output_events};
use clap_sys::ext::log::CLAP_LOG_ERROR;
use clap_sys::id::clap_id;
use std::mem::MaybeUninit;

pub struct ParamInfoWriter<'a> {
    buf: &'a mut MaybeUninit<clap_param_info>,
    is_set: bool,
}

impl ParamInfoWriter<'_> {
    /// # Safety
    ///
    /// The user must ensure the provided pointer is aligned and points to a valid allocation.
    /// However, it doesn't have to be initialized.
    unsafe fn new(raw: *mut clap_param_info) -> Self {
        Self {
            // SAFETY: MaybeUninit<T> and T have same memory representation
            buf: unsafe { &mut *raw.cast() },
            is_set: false,
        }
    }

    #[inline]
    pub fn set(&mut self, info: &ParamInfo) {
        let buf = self.buf.as_mut_ptr();

        // SAFETY: all pointers come from `inner`, which is valid for writes and well-aligned
        unsafe {
            (&raw mut (*buf).id).write(info.id.get());
            (&raw mut (*buf).flags).write(info.flags.bits());
            (&raw mut (*buf).min_value).write(info.min_value);
            (&raw mut (*buf).max_value).write(info.max_value);
            (&raw mut (*buf).default_value).write(info.default_value);
            (&raw mut (*buf).cookie).write(info.cookie.as_raw());

            write_to_array_buf(&raw mut ((*buf).name), info.name);
            write_to_array_buf(&raw mut ((*buf).module), info.module);
        }
        self.is_set = true;
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

impl core::fmt::Write for ParamDisplayWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s = s.as_bytes();
        let requested_len = core::cmp::min(s.len(), self.remaining_len());

        if requested_len > 0 {
            self.buffer[self.cursor_position..self.cursor_position + requested_len]
                .copy_from_slice(&s[..requested_len]);
            self.cursor_position += requested_len;
        }

        Ok(())
    }
}

pub trait PluginMainThreadParams {
    fn count(&mut self) -> u32;
    fn get_info(&mut self, param_index: u32, info: &mut ParamInfoWriter);
    fn get_value(&mut self, param_id: ClapId) -> Option<f64>;
    fn value_to_text(
        &mut self,
        param_id: ClapId,
        value: f64,
        writer: &mut ParamDisplayWriter,
    ) -> core::fmt::Result;
    fn text_to_value(&mut self, param_id: ClapId, text: &CStr) -> Option<f64>;
    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        output_parameter_changes: &mut OutputEvents,
    );
}

pub trait PluginAudioProcessorParams {
    fn flush(
        &mut self,
        input_parameter_changes: &InputEvents,
        output_parameter_changes: &mut OutputEvents,
    );
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn count<P>(plugin: *const clap_plugin) -> u32
where
    for<'a> P: Plugin<MainThread<'a>: PluginMainThreadParams>,
{
    PluginWrapper::<P>::handle(plugin, |p| Ok(p.main_thread().as_mut().count())).unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get_info<P>(
    plugin: *const clap_plugin,
    param_index: u32,
    value: *mut clap_param_info,
) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginMainThreadParams>,
{
    let mut info = ParamInfoWriter::new(value);
    PluginWrapper::<P>::handle(plugin, |p| {
        p.main_thread().as_mut().get_info(param_index, &mut info);
        Ok(())
    })
    .is_some()
        && info.is_set
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get_value<P>(
    plugin: *const clap_plugin,
    param_id: clap_id,
    value: *mut f64,
) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginMainThreadParams>,
{
    let val = PluginWrapper::<P>::handle(plugin, |p| {
        let param_id = ClapId::from_raw(param_id)
            .ok_or(PluginWrapperError::InvalidParameter("Invalid param_id"))?;

        Ok(p.main_thread().as_mut().get_value(param_id))
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

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn value_to_text<P>(
    plugin: *const clap_plugin,
    param_id: clap_id,
    value: f64,
    display: *mut std::os::raw::c_char,
    size: u32,
) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginMainThreadParams>,
{
    let buf = slice_from_external_parts_mut(display as *mut u8, size as usize);
    let mut writer = ParamDisplayWriter::new(buf);
    PluginWrapper::<P>::handle(plugin, |p| {
        let param_id = ClapId::from_raw(param_id)
            .ok_or(PluginWrapperError::InvalidParameter("Invalid param_id"))?;

        p.main_thread()
            .as_mut()
            .value_to_text(param_id, value, &mut writer)
            .map_err(PluginWrapperError::with_severity(CLAP_LOG_ERROR))
    })
    .is_some()
        && writer.finish()
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn text_to_value<P>(
    plugin: *const clap_plugin,
    param_id: clap_id,
    display: *const std::os::raw::c_char,
    value: *mut f64,
) -> bool
where
    for<'a> P: Plugin<MainThread<'a>: PluginMainThreadParams>,
{
    let result = PluginWrapper::<P>::handle(plugin, |p| {
        let param_id = ClapId::from_raw(param_id)
            .ok_or(PluginWrapperError::InvalidParameter("Invalid param_id"))?;

        let display = CStr::from_ptr(display);
        Ok(p.main_thread().as_mut().text_to_value(param_id, display))
    });

    match result {
        Some(Some(val)) => {
            *value = val;
            true
        }
        _ => false,
    }
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn flush<P>(
    plugin: *const clap_plugin,
    input_parameter_changes: *const clap_input_events,
    output_parameter_changes: *const clap_output_events,
) where
    for<'a> P: Plugin<
        MainThread<'a>: PluginMainThreadParams,
        AudioProcessor<'a>: PluginAudioProcessorParams,
    >,
{
    let input_parameter_changes = InputEvents::from_raw(&*input_parameter_changes);
    let output_parameter_changes =
        OutputEvents::from_raw_mut(&mut *(output_parameter_changes as *mut _));

    PluginWrapper::<P>::handle(plugin, |p| {
        if let Ok(mut audio) = p.audio_processor() {
            audio
                .as_mut()
                .flush(input_parameter_changes, output_parameter_changes);
        } else {
            p.main_thread()
                .as_mut()
                .flush(input_parameter_changes, output_parameter_changes);
        }
        Ok(())
    });
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<P> ExtensionImplementation<P> for PluginParams
where
    for<'a> P: Plugin<
        MainThread<'a>: PluginMainThreadParams,
        AudioProcessor<'a>: PluginAudioProcessorParams,
    >,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_params {
            count: Some(count::<P>),
            get_info: Some(get_info::<P>),
            get_value: Some(get_value::<P>),
            value_to_text: Some(value_to_text::<P>),
            text_to_value: Some(text_to_value::<P>),
            flush: Some(flush::<P>),
        });
}

impl HostParams {
    #[inline]
    pub fn rescan(&self, host: &mut HostMainThreadHandle, flags: ParamRescanFlags) {
        if let Some(rescan) = host.use_extension(&self.0).rescan {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { rescan(host.as_raw(), flags.bits()) }
        }
    }

    #[inline]
    pub fn clear(&self, host: &mut HostMainThreadHandle, param_id: ClapId, flags: ParamClearFlags) {
        if let Some(clear) = host.use_extension(&self.0).clear {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { clear(host.as_raw(), param_id.get(), flags.bits()) }
        }
    }

    #[inline]
    pub fn request_flush(&self, host: &HostSharedHandle) {
        if let Some(request_flush) = host.use_extension(&self.0).request_flush {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { request_flush(host.as_raw()) }
        }
    }
}
