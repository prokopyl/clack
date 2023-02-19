use super::*;
use crate::utils::write_to_array_buf;
use clack_plugin::extensions::prelude::*;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

/// Implementation of the Plugin-side of the Audio Ports Configuration extension.
pub trait PluginAudioPortsConfigImplementation {
    /// Returns the number of available [`AudioPortsConfiguration`]s.
    fn count(&self) -> u32;

    /// Retrieves a specific [`AudioPortsConfiguration`] from its index.
    ///
    /// The plugin gets passed a host-provided mutable buffer to write the configuration into, to
    /// avoid any unnecessary allocations.
    fn get(&self, index: u32, writer: &mut AudioPortConfigWriter);

    /// Requests the plugin to change its Audio Ports Configuration to the one with the given ID.
    ///
    /// The plugin *must* be deactivated to call this method.
    ///
    /// # Error
    ///
    /// This method may return an [`AudioPortConfigSelectError`] if the given ID is out of bounds,
    /// or if the plugin declined or failed to change its Audio Ports Configuration.
    fn select(&mut self, config_id: u32) -> Result<(), AudioPortConfigSelectError>;
}

impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginAudioPortsConfig
where
    P::MainThread: PluginAudioPortsConfigImplementation,
{
    #[doc(hidden)]
    const IMPLEMENTATION: &'static Self = &PluginAudioPortsConfig(clap_plugin_audio_ports_config {
        count: Some(count::<P>),
        get: Some(get::<P>),
        select: Some(select::<P>),
    });
}

unsafe extern "C" fn count<'a, P: Plugin<'a>>(plugin: *const clap_plugin) -> u32
where
    P::MainThread: PluginAudioPortsConfigImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| Ok(p.main_thread().as_ref().count() as u32)).unwrap_or(0)
}

unsafe extern "C" fn get<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    index: u32,
    config: *mut clap_audio_ports_config,
) -> bool
where
    P::MainThread: PluginAudioPortsConfigImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        if config.is_null() {
            return Err(PluginWrapperError::NulPtr("clap_audio_ports_config"));
        };

        let mut writer = AudioPortConfigWriter::from_raw(config);
        p.main_thread().as_ref().get(index, &mut writer);
        Ok(writer.is_set)
    })
    .unwrap_or(false)
}

unsafe extern "C" fn select<'a, P: Plugin<'a>>(plugin: *const clap_plugin, config_id: u32) -> bool
where
    P::MainThread: PluginAudioPortsConfigImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        if p.is_active() {
            return Err(PluginWrapperError::DeactivationRequiredForFunction(
                "clap_plugin_audio_ports_config.select",
            ));
        }

        Ok(p.main_thread().as_mut().select(config_id).is_ok())
    })
    .unwrap_or(false)
}

/// An helper struct to write an [`AudioPortsConfiguration`] into the host's provided buffer.
pub struct AudioPortConfigWriter<'a> {
    buf: &'a mut MaybeUninit<clap_audio_ports_config>,
    is_set: bool,
}

impl<'a> AudioPortConfigWriter<'a> {
    #[inline]
    unsafe fn from_raw(raw: *mut clap_audio_ports_config) -> Self {
        Self {
            buf: &mut *raw.cast(),
            is_set: false,
        }
    }

    /// Writes the given [`AudioPortsConfiguration`] into the host's buffer.
    #[inline]
    pub fn write(&mut self, data: &AudioPortsConfiguration) {
        use core::ptr::write;

        let buf = self.buf.as_mut_ptr();

        unsafe {
            write(addr_of_mut!((*buf).id), data.id);
            write_to_array_buf(addr_of_mut!((*buf).name), data.name.as_bytes());

            write(addr_of_mut!((*buf).input_port_count), data.input_port_count);
            write(
                addr_of_mut!((*buf).output_port_count),
                data.output_port_count,
            );

            if let Some(info) = data.main_input {
                write(addr_of_mut!((*buf).has_main_input), true);
                write(
                    addr_of_mut!((*buf).main_input_channel_count),
                    info.channel_count,
                );
                write(
                    addr_of_mut!((*buf).main_input_port_type),
                    info.port_type.0.as_ptr(),
                );
            } else {
                write(addr_of_mut!((*buf).has_main_input), false);
                write(addr_of_mut!((*buf).main_input_channel_count), 0);
                write(
                    addr_of_mut!((*buf).main_input_port_type),
                    core::ptr::null_mut(),
                );
            }

            if let Some(info) = data.main_output {
                write(addr_of_mut!((*buf).has_main_output), true);
                write(
                    addr_of_mut!((*buf).main_output_channel_count),
                    info.channel_count,
                );
                write(
                    addr_of_mut!((*buf).main_output_port_type),
                    info.port_type.0.as_ptr(),
                );
            } else {
                write(addr_of_mut!((*buf).has_main_output), false);
                write(addr_of_mut!((*buf).main_output_channel_count), 0);
                write(
                    addr_of_mut!((*buf).main_output_port_type),
                    core::ptr::null_mut(),
                );
            }
        }

        self.is_set = true;
    }
}

impl HostAudioPortsConfig {
    /// Informs the host that the available Audio Ports Configuration list has changed and needs to
    /// be rescanned.
    #[inline]
    pub fn rescan(&self, host: &mut HostMainThreadHandle) {
        if let Some(rescan) = self.0.rescan {
            unsafe { rescan(host.as_raw()) }
        }
    }
}
