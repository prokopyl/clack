use crate::utils::data_from_array_buf;
use bitflags::bitflags;
use clack_common::extensions::{Extension, PluginExtension};
use clap_sys::ext::audio_ports::*;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::str::Utf8Error;

pub use clap_sys::ext::audio_ports::{CLAP_PORT_MONO, CLAP_PORT_STEREO};

#[repr(C)]
pub struct PluginAudioPorts(
    clap_plugin_audio_ports,
    PhantomData<*const clap_plugin_audio_ports>,
);

bitflags! {
    #[repr(C)]
    pub struct RescanType: u32 {
        const RESCAN_NAMES = CLAP_AUDIO_PORTS_RESCAN_NAMES;
        const RESCAN_FLAGS = CLAP_AUDIO_PORTS_RESCAN_FLAGS;
        const RESCAN_CHANNEL_COUNT = CLAP_AUDIO_PORTS_RESCAN_CHANNEL_COUNT;
        const RESCAN_PORT_TYPE = CLAP_AUDIO_PORTS_RESCAN_PORT_TYPE;
        const RESCAN_IN_PLACE_PAIR = CLAP_AUDIO_PORTS_RESCAN_IN_PLACE_PAIR;
        const RESCAN_LIST = CLAP_AUDIO_PORTS_RESCAN_LIST;
    }
}

bitflags! {
    #[repr(C)]
    pub struct AudioPortFlags: u32 {
        const CLAP_AUDIO_PORT_IS_MAIN = CLAP_AUDIO_PORT_IS_MAIN;
        const CLAP_AUDIO_PORT_SUPPORTS_64BITS = CLAP_AUDIO_PORT_SUPPORTS_64BITS;
        const CLAP_AUDIO_PORT_PREFERS_64BITS = CLAP_AUDIO_PORT_PREFERS_64BITS;
        const CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE = CLAP_AUDIO_PORT_REQUIRES_COMMON_SAMPLE_SIZE;
    }
}

unsafe impl Extension for PluginAudioPorts {
    const IDENTIFIER: *const i8 = CLAP_EXT_AUDIO_PORTS;
    type ExtensionType = PluginExtension;
}

pub struct AudioPortInfoData<'a> {
    pub id: u32, // TODO: ClapId
    pub name: &'a str,
    pub channel_count: u32,
    pub flags: AudioPortFlags,
    pub port_type: Option<&'static CStr>, // TODO: proper port types
                                          // TODO: in_place_pair
}

impl<'a> TryFrom<&'a clap_audio_port_info> for AudioPortInfoData<'a> {
    type Error = Utf8Error;

    fn try_from(other: &'a clap_audio_port_info) -> Result<Self, Self::Error> {
        Ok(Self {
            id: other.id,
            name: std::str::from_utf8(data_from_array_buf(&other.name))?,
            channel_count: other.channel_count,
            flags: AudioPortFlags { bits: other.flags },
            port_type: NonNull::new(other.port_type as *mut _)
                .map(|ptr| unsafe { CStr::from_ptr(ptr.as_ptr()) }),
        })
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use clap_sys::ext::audio_ports::*;
    use std::mem::MaybeUninit;

    use super::*;
    use clack_host::host::PluginHoster;
    use clack_host::instance::PluginInstance;

    impl PluginAudioPorts {
        pub fn count_inputs<'a, H: PluginHoster<'a>>(&self, plugin: &PluginInstance<'a, H>) -> u32 {
            unsafe { (self.0.count)(plugin.raw_instance(), true) }
        }

        pub fn count_outputs<'a, H: PluginHoster<'a>>(
            &self,
            plugin: &PluginInstance<'a, H>,
        ) -> u32 {
            unsafe { (self.0.count)(plugin.raw_instance(), false) }
        }

        fn get<'a, H: PluginHoster<'a>>(
            &self,
            plugin: &PluginInstance<'a, H>,
            index: u32,
            is_input: bool,
        ) -> Option<clap_audio_port_info> {
            let mut info_data: MaybeUninit<clap_audio_port_info> = MaybeUninit::uninit();
            if unsafe {
                (self.0.get)(
                    plugin.raw_instance(),
                    index,
                    is_input,
                    info_data.as_mut_ptr(),
                )
            } {
                Some(unsafe { info_data.assume_init() })
            } else {
                None
            }
        }

        pub fn get_input<'a, H: PluginHoster<'a>>(
            &self,
            plugin: &PluginInstance<'a, H>,
            index: u32,
        ) -> Option<clap_audio_port_info> {
            self.get(plugin, index, true)
        }

        pub fn get_output<'a, H: PluginHoster<'a>>(
            &self,
            plugin: &PluginInstance<'a, H>,
            index: u32,
        ) -> Option<clap_audio_port_info> {
            self.get(plugin, index, false)
        }
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use crate::audio_ports::{AudioPortInfoData, PluginAudioPorts};
    use crate::utils::write_to_array_buf;
    use clack_common::extensions::ExtensionImplementation;
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
                write_to_array_buf(addr_of_mut!((*buf).name), data.name);

                write(addr_of_mut!((*buf).flags), data.flags.bits);
                write(addr_of_mut!((*buf).channel_count), data.channel_count);

                write(
                    addr_of_mut!((*buf).port_type),
                    data.port_type
                        .map(|s| s.as_ptr())
                        .unwrap_or(::core::ptr::null()),
                );

                write(addr_of_mut!((*buf).in_place_pair), u32::MAX); // TODO
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
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
