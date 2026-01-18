use crate::surround::{HostSurround, PluginSurround, SurroundChannel, SurroundChannels};
use clack_host::{
    extensions::{ExtensionImplementation, RawExtensionImplementation, wrapper::HostWrapper},
    host::HostHandlers,
    plugin::PluginMainThreadHandle,
};
use clap_sys::{ext::surround::clap_host_surround, host::clap_host};

impl PluginSurround {
    /// Check if the plugin supports the a surround configuration mask.
    pub fn is_channel_mask_supported(
        &self,
        handle: &mut PluginMainThreadHandle,
        mask: SurroundChannels,
    ) -> bool {
        match handle.use_extension(&self.0).is_channel_mask_supported {
            // SAFETY: This type ensures the function pointer is valid.
            Some(is_channel_mask_supported) => unsafe {
                is_channel_mask_supported(handle.as_raw_ptr(), mask.bits())
            },
            None => false,
        }
    }

    /// Fills the given writer with the surround channel map for the given port, if applicable.
    ///
    /// The buffer should be large enough to hold the channel map for the port (i.e., at least `channel_count` bytes long).
    /// This function should only be called if the port it is called for has `port_type` set to [`AudioPortType::SURROUND`](`crate::audio_ports::AudioPortType::SURROUND`).
    pub fn get_channel_map<'a>(
        &self,
        handle: &mut PluginMainThreadHandle,
        is_input: bool,
        port_index: u32,
        buffer: &'a mut [u8],
    ) -> &'a [SurroundChannel] {
        let buffer_len = match u32::try_from(buffer.len()) {
            Ok(len) => len,
            Err(_) => return &[], // buffer is too large to fit in a u32, so we can't write the channel map
        };

        match handle.use_extension(&self.0).get_channel_map {
            // SAFETY: This type ensures the function pointer is valid.
            Some(get_channel_map) => unsafe {
                let written = get_channel_map(
                    handle.as_raw(),
                    is_input,
                    port_index,
                    buffer.as_mut_ptr() as *mut _,
                    buffer_len,
                );

                SurroundChannel::from_raw_slice(&buffer[..written as usize]).unwrap_or_default()
            },
            None => &[],
        }
    }
}

/// The host-side implementation of the Surround extension.
pub trait HostSurroundImpl {
    /// Notify the host that the surround configuration for one or more ports has changed.
    ///
    /// The channel map can only change when the plugin is de-activated.
    fn changed(&mut self);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H> ExtensionImplementation<H> for HostSurround
where
    for<'a> H: HostHandlers<MainThread<'a>: HostSurroundImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_surround {
            changed: Some(changed::<H>),
        });
}

#[allow(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
unsafe extern "C" fn changed<H>(host: *const clap_host)
where
    for<'a> H: HostHandlers<MainThread<'a>: HostSurroundImpl>,
{
    unsafe {
        HostWrapper::<H>::handle(host, |host| {
            host.main_thread().as_mut().changed();
            Ok(())
        });
    }
}
