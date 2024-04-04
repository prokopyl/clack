use super::*;
use crate::utils::write_to_array_buf;
use clack_common::extensions::RawExtensionImplementation;
use clack_plugin::extensions::prelude::*;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

/// Implementation of the Plugin-side of the Audio Ports Configuration extension.
pub trait PluginAudioPortsConfigImpl {
    /// Returns the number of available [`AudioPortsConfiguration`]s.
    fn count(&mut self) -> u32;

    /// Retrieves a specific [`AudioPortsConfiguration`] from its index.
    ///
    /// The plugin gets passed a host-provided mutable buffer to write the configuration into, to
    /// avoid any unnecessary allocations.
    fn get(&mut self, index: u32, writer: &mut AudioPortConfigWriter);

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

impl<P: Plugin> ExtensionImplementation<P> for PluginAudioPortsConfig
where
    for<'a> P::MainThread<'a>: PluginAudioPortsConfigImpl,
{
    #[doc(hidden)]
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_plugin_audio_ports_config {
            count: Some(count::<P>),
            get: Some(get::<P>),
            select: Some(select::<P>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn count<P: Plugin>(plugin: *const clap_plugin) -> u32
where
    for<'a> P::MainThread<'a>: PluginAudioPortsConfigImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| Ok(p.main_thread().as_mut().count())).unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn get<P: Plugin>(
    plugin: *const clap_plugin,
    index: u32,
    config: *mut clap_audio_ports_config,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginAudioPortsConfigImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        if config.is_null() {
            return Err(PluginWrapperError::NulPtr("clap_audio_ports_config"));
        };

        let mut writer = AudioPortConfigWriter::from_raw(config);
        p.main_thread().as_mut().get(index, &mut writer);
        Ok(writer.is_set)
    })
    .unwrap_or(false)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn select<P: Plugin>(plugin: *const clap_plugin, config_id: u32) -> bool
where
    for<'a> P::MainThread<'a>: PluginAudioPortsConfigImpl,
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

/// A helper struct to write an [`AudioPortsConfiguration`] into the host's provided buffer.
pub struct AudioPortConfigWriter<'a> {
    buf: &'a mut MaybeUninit<clap_audio_ports_config>,
    is_set: bool,
}

impl<'a> AudioPortConfigWriter<'a> {
    /// # Safety
    ///
    /// The user must ensure the provided pointer is aligned and points to a valid allocation.
    /// However, it doesn't have to be initialized.
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

        // SAFETY: all pointers come from `buf`, which is valid for writes and well-aligned
        unsafe {
            write(addr_of_mut!((*buf).id), data.id);
            write_to_array_buf(addr_of_mut!((*buf).name), data.name);

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
                    info.port_type
                        .map(|t| t.0.as_ptr())
                        .unwrap_or(core::ptr::null()),
                );
            } else {
                write(addr_of_mut!((*buf).has_main_input), false);
                write(addr_of_mut!((*buf).main_input_channel_count), 0);
                write(addr_of_mut!((*buf).main_input_port_type), core::ptr::null());
            }

            if let Some(info) = data.main_output {
                write(addr_of_mut!((*buf).has_main_output), true);
                write(
                    addr_of_mut!((*buf).main_output_channel_count),
                    info.channel_count,
                );
                write(
                    addr_of_mut!((*buf).main_output_port_type),
                    info.port_type
                        .map(|t| t.0.as_ptr())
                        .unwrap_or(core::ptr::null()),
                );
            } else {
                write(addr_of_mut!((*buf).has_main_output), false);
                write(addr_of_mut!((*buf).main_output_channel_count), 0);
                write(
                    addr_of_mut!((*buf).main_output_port_type),
                    core::ptr::null(),
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
        if let Some(rescan) = host.use_extension(&self.0).rescan {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { rescan(host.as_raw()) }
        }
    }
}
