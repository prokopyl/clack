use super::*;
use clack_host::extensions::prelude::*;
use std::mem::MaybeUninit;

/// A host-provided buffer for the plugin to write an Audio Port Configuration in.
#[derive(Clone)]
pub struct AudioPortsConfigBuffer {
    inner: MaybeUninit<clap_audio_ports_config>,
}

impl Default for AudioPortsConfigBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPortsConfigBuffer {
    /// Creates an uninitialized Audio Port Configuration buffer.
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
        }
    }
}

impl PluginAudioPortsConfig {
    /// Returns the number of available [`AudioPortsConfiguration`]s.
    pub fn count(&self, plugin: &mut PluginMainThreadHandle) -> usize {
        match self.0.count {
            None => 0,
            Some(count) => unsafe { count(plugin.as_raw()) as usize },
        }
    }

    /// Retrieves a specific [`AudioPortsConfiguration`] from its index.
    ///
    /// The plugin gets passed a mutable buffer to write the configuration into, to avoid any
    /// unnecessary allocations.
    pub fn get<'b>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        index: usize,
        buffer: &'b mut AudioPortsConfigBuffer,
    ) -> Option<AudioPortsConfiguration<'b>> {
        let success =
            unsafe { (self.0.get?)(plugin.as_raw(), index as u32, buffer.inner.as_mut_ptr()) };

        if success {
            Some(unsafe { AudioPortsConfiguration::from_raw(buffer.inner.assume_init_ref()) })
        } else {
            None
        }
    }

    /// Requests the plugin to change its Audio Ports Configuration to the one with the given ID.
    ///
    /// The plugin *must* be deactivated to call this method.
    ///
    /// # Error
    ///
    /// This method may return an [`AudioPortConfigSelectError`] if the given ID is out of bounds,
    /// or if the plugin declined or failed to change its Audio Ports Configuration.
    #[inline]
    pub fn select(
        &self,
        plugin: &mut PluginMainThreadHandle,
        configuration_id: u32,
    ) -> Result<(), AudioPortConfigSelectError> {
        let success = unsafe {
            self.0.select.ok_or(AudioPortConfigSelectError)?(plugin.as_raw(), configuration_id)
        };

        match success {
            true => Ok(()),
            false => Err(AudioPortConfigSelectError),
        }
    }
}

/// Implementation of the Host-side of the Audio Ports Configuration extension.
pub trait HostAudioPortsConfigImpl {
    /// Informs the host that the available Audio Ports Configuration list has changed and needs to
    /// be rescanned.
    fn rescan(&mut self);
}

impl<H: Host> ExtensionImplementation<H> for HostAudioPortsConfig
where
    for<'h> <H as Host>::MainThread<'h>: HostAudioPortsConfigImpl,
{
    #[doc(hidden)]
    const IMPLEMENTATION: &'static Self = &HostAudioPortsConfig(clap_host_audio_ports_config {
        rescan: Some(rescan::<H>),
    });
}

unsafe extern "C" fn rescan<H: Host>(host: *const clap_host)
where
    for<'a> <H as Host>::MainThread<'a>: HostAudioPortsConfigImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread().as_mut().rescan();

        Ok(())
    });
}
