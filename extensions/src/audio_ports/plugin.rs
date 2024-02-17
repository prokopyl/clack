use crate::audio_ports::{AudioPortInfoData, HostAudioPorts, PluginAudioPorts, RescanType};
use crate::utils::write_to_array_buf;
use clack_plugin::extensions::prelude::*;
use clap_sys::ext::audio_ports::{clap_audio_port_info, clap_plugin_audio_ports};
use clap_sys::id::CLAP_INVALID_ID;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

pub struct AudioPortInfoWriter<'a> {
    pub(crate) buf: &'a mut MaybeUninit<clap_audio_port_info>,
    pub(crate) is_set: bool,
}

impl<'a> AudioPortInfoWriter<'a> {
    #[inline]
    unsafe fn from_raw(raw: *mut clap_audio_port_info) -> Self {
        Self {
            buf: &mut *raw.cast(),
            is_set: false,
        }
    }

    #[inline]
    pub fn set(&mut self, data: &AudioPortInfoData) {
        use core::ptr::write;

        let buf = self.buf.as_mut_ptr();

        unsafe {
            write(addr_of_mut!((*buf).id), data.id);
            write_to_array_buf(addr_of_mut!((*buf).name), data.name);

            write(addr_of_mut!((*buf).flags), data.flags.bits);
            write(addr_of_mut!((*buf).channel_count), data.channel_count);

            write(
                addr_of_mut!((*buf).port_type),
                data.port_type
                    .map(|s| s.0.as_ptr())
                    .unwrap_or(core::ptr::null()),
            );

            write(
                addr_of_mut!((*buf).in_place_pair),
                data.in_place_pair.unwrap_or(CLAP_INVALID_ID),
            );
        }

        self.is_set = true;
    }
}

pub trait PluginAudioPortsImpl {
    fn count(&self, is_input: bool) -> u32;
    fn get(&self, is_input: bool, index: u32, writer: &mut AudioPortInfoWriter);
}

impl<P: Plugin> ExtensionImplementation<P> for PluginAudioPorts
where
    for<'a> P::MainThread<'a>: PluginAudioPortsImpl,
{
    const IMPLEMENTATION: &'static Self = &PluginAudioPorts(
        clap_plugin_audio_ports {
            count: Some(count::<P>),
            get: Some(get::<P>),
        },
        PhantomData,
    );
}

unsafe extern "C" fn count<P: Plugin>(plugin: *const clap_plugin, is_input: bool) -> u32
where
    for<'a> P::MainThread<'a>: PluginAudioPortsImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| Ok(p.main_thread().as_ref().count(is_input)))
        .unwrap_or(0)
}

unsafe extern "C" fn get<P: Plugin>(
    plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginAudioPortsImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        if info.is_null() {
            return Err(PluginWrapperError::NulPtr("clap_audio_port_info"));
        };

        let mut writer = AudioPortInfoWriter::from_raw(info);
        p.main_thread().as_ref().get(is_input, index, &mut writer);
        Ok(writer.is_set)
    })
    .unwrap_or(false)
}

impl HostAudioPorts {
    #[inline]
    pub fn is_rescan_flag_supported(&self, host: &HostMainThreadHandle, flag: RescanType) -> bool {
        match self.0.is_rescan_flag_supported {
            None => false,
            Some(supported) => unsafe { supported(host.as_raw(), flag.bits) },
        }
    }

    #[inline]
    pub fn rescan(&self, host: &mut HostMainThreadHandle, flag: RescanType) {
        if let Some(rescan) = self.0.rescan {
            unsafe { rescan(host.as_raw(), flag.bits) }
        }
    }
}
