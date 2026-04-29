use super::*;
use clack_common::events::io::{InputEvents, OutputEvents};
use clack_host::extensions::prelude::*;
use std::mem::MaybeUninit;

/// A buffer in which plugins may write parameter information into.
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
    /// Creates a new, empty [`ParamInfoBuffer`]  for the plugin to write parameter information into.
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::zeroed(),
        }
    }
}

impl PluginParams {
    /// Returns the total number of parameters the plugin exposes.
    pub fn count(&self, plugin: &mut PluginMainThreadHandle) -> u32 {
        match plugin.use_extension(&self.0).count {
            None => 0,
            // SAFETY: This type ensures the function pointer is valid.
            Some(count) => unsafe { count(plugin.as_raw()) },
        }
    }

    /// Gets the metadata for a parameter by its index.
    ///
    /// The host calls this to learn about a parameter’s identity, range, name,
    /// and other properties. The implementation should write the parameter’s
    /// metadata into the provided `info` writer.
    ///
    /// # Arguments
    ///
    /// * `index`: The index of the parameter to query. Must be less than
    ///   the value returned by `count()`.
    /// * `info`: A writer to populate with the parameter’s metadata.
    ///
    /// # Return
    ///
    /// Returns `true` on success, or `false` if `index` is out of bounds.
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

    /// Gets the current value of a parameter by its ID.
    ///
    /// The host calls this to read a parameter’s state. The implementation
    /// should return the parameter’s current plain value.
    ///
    /// # Arguments
    ///
    /// * `param_id`: The ID of the parameter to query.
    ///
    /// # Return
    ///
    /// Returns the current value of the parameter, or `None` if the ID is invalid.
    pub fn get_value(&self, plugin: &mut PluginMainThreadHandle, param_id: ClapId) -> Option<f64> {
        let mut value = 0.0;
        // SAFETY: This type ensures the function pointer is valid.
        let valid = unsafe {
            plugin.use_extension(&self.0).get_value?(plugin.as_raw(), param_id.get(), &mut value)
        };

        if valid { Some(value) } else { None }
    }

    /// Converts a parameter’s plain value to a human-readable string.
    ///
    /// The host uses this to display parameter values in a user-friendly format,
    /// such as "440.0 Hz" instead of just "440.0".
    ///
    /// Note that this method will always reset the buffer to all-zeros before each use, for safety
    /// reasons in case the plugin implementation misbehaves.
    /// Clearing the buffer before passing it to this method is therefore redundant.
    ///
    /// # Arguments
    ///
    /// * `param_id`: The ID of the parameter.
    /// * `value`: The plain value to format.
    /// * `buffer`: A buffer to write the formatted string into.
    ///
    /// # Return
    ///
    /// Returns `Ok(())` on success, or `Err` if formatting fails.
    pub fn value_to_text<'b>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        param_id: ClapId,
        value: f64,
        buffer: &'b mut [u8],
    ) -> Result<&'b mut [u8], core::fmt::Error> {
        let Some(value_to_text) = plugin.use_extension(&self.0).value_to_text else {
            return Err(core::fmt::Error);
        };
        let len = u32::try_from(buffer.len()).unwrap_or(u32::MAX);

        buffer.fill(0);

        // SAFETY: This type ensures the function pointer is valid.
        let valid = unsafe {
            value_to_text(
                plugin.as_raw(),
                param_id.get(),
                value,
                buffer.as_mut_ptr() as *mut _,
                len,
            )
        };

        if !valid {
            return Err(core::fmt::Error);
        }

        // If no nul byte found, we take the entire buffer
        let buffer_total_len = buffer.iter().position(|b| *b == 0).unwrap_or(len as usize);
        Ok(&mut buffer[..buffer_total_len])
    }

    /// Converts a human-readable string back to a parameter’s plain value.
    ///
    /// The host uses this to handle user text input for parameter values.
    ///
    /// # Arguments
    ///
    /// * `param_id`: The ID of the parameter.
    /// * `text`: The text to parse.
    ///
    /// # Return
    ///
    /// Returns the parsed value, or `None` if parsing fails or the ID is invalid.
    pub fn text_to_value(
        &self,
        plugin: &mut PluginMainThreadHandle,
        param_id: ClapId,
        text: &CStr,
    ) -> Option<f64> {
        let mut value = 0.0;

        // SAFETY: This type ensures the function pointer is valid.
        let valid = unsafe {
            plugin.use_extension(&self.0).text_to_value?(
                plugin.as_raw(),
                param_id.get(),
                text.as_ptr(),
                &mut value,
            )
        };

        if valid { Some(value) } else { None }
    }

    /// Flushes pending parameter changes between the host and plugin.
    ///
    /// This method is called by the host to synchronize parameter values in
    /// either direction. It receives incoming changes via `input_parameter_changes`
    /// and allows the plugin to send outgoing changes via `output_parameter_changes`.
    ///
    /// This is typically called when the plugin is not actively processing audio,
    /// but can also be used for parameter automation without audio playback.
    ///
    /// Note that if the plugin is processing, then the `process()` call will already achieve the
    /// parameter update (bidirectional), so a call to flush isn't required.
    /// Also be aware that the plugin may use the sample offset in `process()`, while this information would be
    /// lost within `flush()`.
    ///
    /// # Arguments
    ///
    /// * `input_parameter_changes`: A reader for incoming parameter change events.
    /// * `output_parameter_changes`: A writer for outgoing parameter change events.
    ///
    ///
    pub fn flush(
        &self,
        plugin: &mut InactivePluginMainThreadHandle,
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

    /// Flushes a set of parameter changes when the plugin is active.
    /// This method cannot be called concurrently with `process`, which is statically guaranteed by the [`PluginAudioProcessorHandle`] type.
    ///
    /// Note: if the plugin is processing, then the process() call will already
    /// achieve the parameter update (bidirectional), so a call to flush isn't
    /// required, also be aware that the plugin may use the sample offset in
    /// process(), while this information would be lost within flush().
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

/// Implementation of the thread-safe Host Params extension operations.
pub trait HostParamsImplShared {
    /// Plugin requested a parameter flush.
    ///
    /// The host must then schedule a call to either:
    /// - [`PluginParams::flush`] or [`PluginParams::flush_active`]
    /// - the process callback
    ///
    /// This function is always safe to use, but should not be called from the plugin's audio thread,
    /// as it would already be within `process()` or `flush()`.
    fn request_flush(&self);
}

/// Implementation of the main-thread Host Params extension operations.
pub trait HostParamsImplMainThread {
    /// Rescan the full list of parameters, according to the given `flags`.
    /// See [`ParamRescanFlags`] for more details.
    fn rescan(&mut self, flags: ParamRescanFlags);
    /// Clears references (such as automation or modulation) to a parameter (identified by `param_id`), according to the given `flags`.
    /// See [`ParamClearFlags`] for more details.
    fn clear(&mut self, param_id: ClapId, flags: ParamClearFlags);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H> ExtensionImplementation<H> for HostParams
where
    for<'a> H:
        HostHandlers<Shared<'a>: HostParamsImplShared, MainThread<'a>: HostParamsImplMainThread>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_params {
            rescan: Some(rescan::<H>),
            clear: Some(clear::<H>),
            request_flush: Some(request_flush::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn rescan<H>(host: *const clap_host, flags: clap_param_rescan_flags)
where
    for<'a> H: HostHandlers<MainThread<'a>: HostParamsImplMainThread>,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(ParamRescanFlags::from_bits_truncate(flags));

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn clear<H>(host: *const clap_host, param_id: u32, flags: clap_param_clear_flags)
where
    for<'a> H: HostHandlers<MainThread<'a>: HostParamsImplMainThread>,
{
    HostWrapper::<H>::handle(host, |host| {
        let param_id = ClapId::from_raw(param_id)
            .ok_or(HostWrapperError::InvalidParameter("Invalid param_id"))?;
        host.main_thread()
            .as_mut()
            .clear(param_id, ParamClearFlags::from_bits_truncate(flags));

        Ok(())
    });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn request_flush<H>(host: *const clap_host)
where
    for<'a> H: HostHandlers<Shared<'a>: HostParamsImplShared>,
{
    HostWrapper::<H>::handle(host, |host| {
        host.shared().request_flush();

        Ok(())
    });
}
