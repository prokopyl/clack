use bitflags::bitflags;
use clack_common::extensions::{Extension, PluginExtension};
use clap_sys::ext::audio_ports::*;
use std::marker::PhantomData;

#[repr(C)]
pub struct PluginAudioPorts(
    clap_plugin_audio_ports,
    PhantomData<*const clap_plugin_audio_ports>,
);

#[derive(Copy, Clone, Debug)]
pub enum SampleSize {
    F32,
    F64,
}

impl SampleSize {
    #[inline]
    pub fn bit_size(self) -> u32 {
        match self {
            SampleSize::F32 => 32,
            SampleSize::F64 => 64,
        }
    }
}

bitflags! {
    #[repr(C)]
    pub struct RescanType: u32 {
        const RESCAN_NAMES = CLAP_AUDIO_PORTS_RESCAN_NAMES;
        const RESCAN_ALL = CLAP_AUDIO_PORTS_RESCAN_ALL;
    }
}

unsafe impl Extension for PluginAudioPorts {
    const IDENTIFIER: *const u8 = CLAP_EXT_AUDIO_PORTS as *const _;
    type ExtensionType = PluginExtension;
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use crate::audio_ports::{PluginAudioPorts, SampleSize};
    use clack_common::extensions::ExtensionImplementation;
    use clack_common::ports::ChannelMap;
    use clack_plugin::plugin::wrapper::{PluginWrapper, PluginWrapperError};
    use clack_plugin::plugin::Plugin;
    use clap_sys::ext::audio_ports::{clap_audio_port_info, clap_plugin_audio_ports};
    use clap_sys::plugin::clap_plugin;
    use clap_sys::string_sizes::CLAP_NAME_SIZE;
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
        #[allow(clippy::too_many_arguments)]
        pub fn set(
            &mut self,
            id: u32,
            name: &str,
            channel_count: u32,
            channel_map: ChannelMap,
            sample_size: SampleSize,
            is_main: bool,
            is_cv: bool,
            in_place: bool,
        ) {
            let buf = self.buf.as_mut_ptr();
            unsafe { core::ptr::write(addr_of_mut!((*buf).id), id) };
            unsafe { core::ptr::write(addr_of_mut!((*buf).channel_count), channel_count) };
            unsafe { core::ptr::write(addr_of_mut!((*buf).channel_map), channel_map.to_raw()) };
            unsafe { core::ptr::write(addr_of_mut!((*buf).sample_size), sample_size.bit_size()) };
            unsafe { core::ptr::write(addr_of_mut!((*buf).is_main), is_main) };
            unsafe { core::ptr::write(addr_of_mut!((*buf).is_cv), is_cv) };
            unsafe { core::ptr::write(addr_of_mut!((*buf).in_place), in_place) };

            let dst_name = unsafe {
                &mut *(addr_of_mut!((*buf).name) as *mut _
                    as *mut [MaybeUninit<u8>; CLAP_NAME_SIZE])
            };
            let src_name: &[MaybeUninit<u8>] = unsafe { core::mem::transmute(name.as_bytes()) };
            let len = src_name.len().min(dst_name.len() - 1);
            let dst_name = &mut dst_name[..len];
            let src_name = &src_name[..len];
            dst_name.copy_from_slice(src_name);
            dst_name[len] = MaybeUninit::new(0);

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
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
