use super::*;
use clack_host::extensions::prelude::*;
use core::mem::MaybeUninit;

/// A scratch buffer for the plugin to write audio port metadata to.
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
    /// Get an empty buffer for the plugin to write audio port metadata into.
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::zeroed(),
        }
    }

    pub(crate) fn as_raw(&mut self) -> *mut clap_audio_port_info {
        self.inner.as_mut_ptr()
    }

    /// # Safety
    /// The user must ensure that the buffer has been properly initialized by the plugin.
    pub(crate) unsafe fn assume_init(&self) -> Option<AudioPortInfo<'_>> {
        // SAFETY: the caller ensures the buffer is initialized
        unsafe { AudioPortInfo::from_raw(self.inner.assume_init_ref()) }
    }
}

impl PluginAudioPorts {
    /// Returns number of audio ports, for either input or output
    pub fn count(&self, plugin: &mut PluginMainThreadHandle, is_input: bool) -> u32 {
        match plugin.use_extension(&self.0).count {
            None => 0,
            // SAFETY: This type ensures the function pointer is valid.
            Some(count) => unsafe { count(plugin.as_raw(), is_input) },
        }
    }

    /// Gets information about an audio port by its index, for either input or output.
    pub fn get<'b>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        index: u32,
        is_input: bool,
        buffer: &'b mut AudioPortInfoBuffer,
    ) -> Option<AudioPortInfo<'b>> {
        // SAFETY: This type ensures the function pointer is valid.
        let success = unsafe {
            plugin.use_extension(&self.0).get?(plugin.as_raw(), index, is_input, buffer.as_raw())
        };

        if success {
            // SAFETY: we checked if the buffer was successfully written to
            unsafe { buffer.assume_init() }
        } else {
            None
        }
    }
}

/// Implementation of the Host-side of the Audio Ports extension.
pub trait HostAudioPortsImpl {
    /// Checks if the host allows a plugin to change a given aspect of the audio ports definition.
    fn is_rescan_flag_supported(&self, flag: AudioPortRescanFlags) -> bool;

    /// Rescan the full list of audio ports according to the flags.
    /// It is illegal to ask the host to rescan with a flag that is not supported (see [`is_rescan_flag_supported`](Self::is_rescan_flag_supported)).
    /// Certain flags require the plugin to be de-activated.
    fn rescan(&mut self, flags: AudioPortRescanFlags);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H> ExtensionImplementation<H> for HostAudioPorts
where
    H: for<'a> HostHandlers<MainThread<'a>: HostAudioPortsImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_audio_ports {
            is_rescan_flag_supported: Some(is_rescan_flag_supported::<H>),
            rescan: Some(rescan::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn is_rescan_flag_supported<H>(host: *const clap_host, flag: u32) -> bool
where
    H: for<'a> HostHandlers<MainThread<'a>: HostAudioPortsImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        Ok(host
            .main_thread()
            .as_ref()
            .is_rescan_flag_supported(AudioPortRescanFlags::from_bits_truncate(flag)))
    })
    .unwrap_or(false)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn rescan<H>(host: *const clap_host, flags: u32)
where
    H: for<'a> HostHandlers<MainThread<'a>: HostAudioPortsImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(AudioPortRescanFlags::from_bits_truncate(flags));

        Ok(())
    });
}
