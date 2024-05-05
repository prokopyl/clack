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
        // SAFETY: This type ensures the function pointer is valid.
        match plugin.use_extension(&self.0).count {
            None => 0,
            // SAFETY: This type ensures the function pointer is valid.
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
        // SAFETY: This type ensures the function pointer is valid.
        let success = unsafe {
            plugin.use_extension(&self.0).get?(
                plugin.as_raw(),
                index as u32,
                buffer.inner.as_mut_ptr(),
            )
        };

        if success {
            // SAFETY: we checked if the buffer was successfully written to
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
        // SAFETY: This type ensures the function pointer is valid.
        let success = unsafe {
            plugin
                .use_extension(&self.0)
                .select
                .ok_or(AudioPortConfigSelectError)?(plugin.as_raw(), configuration_id)
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

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H: HostHandlers> ExtensionImplementation<H> for HostAudioPortsConfig
where
    for<'h> <H as HostHandlers>::MainThread<'h>: HostAudioPortsConfigImpl,
{
    #[doc(hidden)]
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_audio_ports_config {
            rescan: Some(rescan::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn rescan<H: HostHandlers>(host: *const clap_host)
where
    for<'a> <H as HostHandlers>::MainThread<'a>: HostAudioPortsConfigImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread().as_mut().rescan();

        Ok(())
    });
}
