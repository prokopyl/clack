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
            inner: MaybeUninit::uninit(),
        }
    }
}

impl PluginAudioPorts {
    pub fn count(&self, plugin: &PluginMainThreadHandle, is_input: bool) -> u32 {
        match self.0.count {
            None => 0,
            Some(count) => unsafe { count(plugin.as_raw(), is_input) },
        }
    }

    pub fn get<'b>(
        &self,
        plugin: &PluginMainThreadHandle,
        index: u32,
        is_input: bool,
        buffer: &'b mut AudioPortInfoBuffer,
    ) -> Option<AudioPortInfoData<'b>> {
        let success =
            unsafe { (self.0.get?)(plugin.as_raw(), index, is_input, buffer.inner.as_mut_ptr()) };

        if success {
            Some(unsafe { AudioPortInfoData::from_raw(buffer.inner.assume_init_ref()) })
        } else {
            None
        }
    }
}

pub trait HostAudioPortsImpl {
    fn is_rescan_flag_supported(&self, flag: RescanType) -> bool;
    fn rescan(&mut self, flag: RescanType);
}

impl<H: for<'h> Host<'h>> ExtensionImplementation<H> for HostAudioPorts
where
    for<'h> <H as Host<'h>>::MainThread: HostAudioPortsImpl,
{
    const IMPLEMENTATION: &'static Self = &HostAudioPorts(
        clap_host_audio_ports {
            is_rescan_flag_supported: Some(is_rescan_flag_supported::<H>),
            rescan: Some(rescan::<H>),
        },
        PhantomData,
    );
}

unsafe extern "C" fn is_rescan_flag_supported<H: for<'a> Host<'a>>(
    host: *const clap_host,
    flag: u32,
) -> bool
where
    for<'a> <H as Host<'a>>::MainThread: HostAudioPortsImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        Ok(host
            .main_thread()
            .as_ref()
            .is_rescan_flag_supported(RescanType::from_bits_truncate(flag)))
    })
    .unwrap_or(false)
}

unsafe extern "C" fn rescan<H: for<'a> Host<'a>>(host: *const clap_host, flag: u32)
where
    for<'a> <H as Host<'a>>::MainThread: HostAudioPortsImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(RescanType::from_bits_truncate(flag));

        Ok(())
    });
}
