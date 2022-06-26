use crate::audio_ports::{AudioPortInfoData, HostAudioPorts, PluginAudioPorts, RescanType};
use crate::utils::write_to_array_buf;
use clack_common::extensions::ExtensionImplementation;
use clack_plugin::host::HostMainThreadHandle;
use clack_plugin::plugin::wrapper::{PluginWrapper, PluginWrapperError};
use clack_plugin::plugin::Plugin;
use clap_sys::ext::audio_ports::{clap_audio_port_info, clap_plugin_audio_ports};
use clap_sys::plugin::clap_plugin;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

pub struct AudioPortInfoWriter<'a> {
    buf: &'a mut MaybeUninit<clap_audio_port_info>,
    is_set: bool,
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
            write_to_array_buf(addr_of_mut!((*buf).name), data.name.to_bytes_with_nul());

            write(addr_of_mut!((*buf).flags), data.flags.bits);
            write(addr_of_mut!((*buf).channel_count), data.channel_count);

            write(
                addr_of_mut!((*buf).port_type),
                data.port_type
                    .map(|s| s.0.as_ptr())
                    .unwrap_or(core::ptr::null()),
            );

            write(addr_of_mut!((*buf).in_place_pair), data.in_place_pair);
        }

        self.is_set = true;
    }
}

pub trait PluginAudioPortsImplementation {
    fn count(&self, is_input: bool) -> usize;
    fn get(&self, is_input: bool, index: usize, writer: &mut AudioPortInfoWriter);
}

impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginAudioPorts
where
    P::MainThread: PluginAudioPortsImplementation,
{
    const IMPLEMENTATION: &'static Self = &PluginAudioPorts(
        clap_plugin_audio_ports {
            count: count::<P>,
            get: get::<P>,
        },
        PhantomData,
    );
}

unsafe extern "C" fn count<'a, P: Plugin<'a>>(plugin: *const clap_plugin, is_input: bool) -> u32
where
    P::MainThread: PluginAudioPortsImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        Ok(p.main_thread().as_ref().count(is_input) as u32)
    })
    .unwrap_or(0)
}

unsafe extern "C" fn get<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_audio_port_info,
) -> bool
where
    P::MainThread: PluginAudioPortsImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        if info.is_null() {
            return Err(PluginWrapperError::NulPtr("clap_audio_port_info"));
        };

        let mut writer = AudioPortInfoWriter::from_raw(info);
        p.main_thread()
            .as_ref()
            .get(is_input, index as usize, &mut writer);
        Ok(writer.is_set)
    })
    .unwrap_or(false)
}

impl HostAudioPorts {
    #[inline]
    pub fn is_rescan_flag_supported(&self, host: &HostMainThreadHandle, flag: RescanType) -> bool {
        unsafe { (self.0.is_rescan_flag_supported)(host.as_raw(), flag.bits) }
    }

    #[inline]
    pub fn rescan(&self, host: &mut HostMainThreadHandle, flag: RescanType) {
        unsafe { (self.0.rescan)(host.as_raw(), flag.bits) }
    }
}
