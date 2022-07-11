use super::*;
use crate::utils::write_to_array_buf;
use clack_common::extensions::ExtensionImplementation;
use clack_plugin::host::HostMainThreadHandle;
use clack_plugin::plugin::wrapper::{PluginWrapper, PluginWrapperError};
use clack_plugin::plugin::Plugin;
use clap_sys::plugin::clap_plugin;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

pub struct NotePortInfoWriter<'a> {
    buf: &'a mut MaybeUninit<clap_note_port_info>,
    is_set: bool,
}

impl<'a> NotePortInfoWriter<'a> {
    #[inline]
    unsafe fn from_raw(raw: *mut clap_note_port_info) -> Self {
        Self {
            buf: &mut *raw.cast(),
            is_set: false,
        }
    }

    #[inline]
    pub fn set(&mut self, data: &NotePortInfoData) {
        use core::ptr::write;

        let buf = self.buf.as_mut_ptr();

        unsafe {
            write(addr_of_mut!((*buf).id), data.id);
            write_to_array_buf(addr_of_mut!((*buf).name), data.name.to_bytes_with_nul());

            write(
                addr_of_mut!((*buf).supported_dialects),
                data.supported_dialects.bits,
            );
            write(
                addr_of_mut!((*buf).preferred_dialect),
                data.preferred_dialect.map(|d| d as u32).unwrap_or(0),
            );
        }

        self.is_set = true;
    }
}

pub trait PluginNotePortsImplementation {
    fn count(&self, is_input: bool) -> usize;
    fn get(&self, is_input: bool, index: usize, writer: &mut NotePortInfoWriter);
}

impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginNotePorts
where
    P::MainThread: PluginNotePortsImplementation,
{
    const IMPLEMENTATION: &'static Self = &PluginNotePorts(
        clap_plugin_note_ports {
            count: Some(count::<P>),
            get: Some(get::<P>),
        },
        PhantomData,
    );
}

unsafe extern "C" fn count<'a, P: Plugin<'a>>(plugin: *const clap_plugin, is_input: bool) -> u32
where
    P::MainThread: PluginNotePortsImplementation,
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
    info: *mut clap_note_port_info,
) -> bool
where
    P::MainThread: PluginNotePortsImplementation,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        if info.is_null() {
            return Err(PluginWrapperError::NulPtr("clap_note_port_info"));
        };

        let mut writer = NotePortInfoWriter::from_raw(info);
        p.main_thread()
            .as_ref()
            .get(is_input, index as usize, &mut writer);
        Ok(writer.is_set)
    })
    .unwrap_or(false)
}

impl HostNotePorts {
    #[inline]
    pub fn supported_dialects(&self, host: &HostMainThreadHandle) -> NoteDialects {
        match self.0.supported_dialects {
            None => NoteDialects::empty(),
            Some(supported) => {
                NoteDialects::from_bits_truncate(unsafe { supported(host.as_raw()) })
            }
        }
    }

    #[inline]
    pub fn rescan(&self, host: &mut HostMainThreadHandle, flags: NotePortRescanFlags) {
        if let Some(rescan) = self.0.rescan {
            unsafe { rescan(host.as_raw(), flags.bits) }
        }
    }
}
