use super::*;
use clack_host::extensions::prelude::*;
use core::mem::MaybeUninit;

#[derive(Clone)]
pub struct AudioPortInfoBuffer {
    inner: MaybeUninit<clap_audio_port_info>,
}

impl Default for AudioPortInfoBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl AudioPortInfoBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::zeroed(),
        }
    }
}

impl PluginAudioPorts {
    pub fn count(&self, plugin: &mut PluginMainThreadHandle, is_input: bool) -> u32 {
        match plugin.use_extension(&self.0).count {
            None => 0,
            // SAFETY: This type ensures the function pointer is valid.
            Some(count) => unsafe { count(plugin.as_raw(), is_input) },
        }
    }

    pub fn get<'b>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        index: u32,
        is_input: bool,
        buffer: &'b mut AudioPortInfoBuffer,
    ) -> Option<AudioPortInfo<'b>> {
        // SAFETY: This type ensures the function pointer is valid.
        let success = unsafe {
            plugin.use_extension(&self.0).get?(
                plugin.as_raw(),
                index,
                is_input,
                buffer.inner.as_mut_ptr(),
            )
        };

        if success {
            // SAFETY: we checked if the buffer was successfully written to
            Some(unsafe { AudioPortInfo::from_raw(buffer.inner.assume_init_ref()) })
        } else {
            None
        }
    }
}

pub trait HostAudioPortsImpl {
    fn is_rescan_flag_supported(&self, flag: RescanType) -> bool;
    fn rescan(&mut self, flag: RescanType);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H: HostHandlers> ExtensionImplementation<H> for HostAudioPorts
where
    for<'h> <H as HostHandlers>::MainThread<'h>: HostAudioPortsImpl,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_audio_ports {
            is_rescan_flag_supported: Some(is_rescan_flag_supported::<H>),
            rescan: Some(rescan::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn is_rescan_flag_supported<H: HostHandlers>(
    host: *const clap_host,
    flag: u32,
) -> bool
where
    for<'a> <H as HostHandlers>::MainThread<'a>: HostAudioPortsImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        Ok(host
            .main_thread()
            .as_ref()
            .is_rescan_flag_supported(RescanType::from_bits_truncate(flag)))
    })
    .unwrap_or(false)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn rescan<H: HostHandlers>(host: *const clap_host, flag: u32)
where
    for<'a> <H as HostHandlers>::MainThread<'a>: HostAudioPortsImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(RescanType::from_bits_truncate(flag));

        Ok(())
    });
}
