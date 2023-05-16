use super::*;
use crate::utils::write_to_array_buf;
use clack_plugin::extensions::prelude::*;
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
            write_to_array_buf(addr_of_mut!((*buf).name), data.name);

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

pub trait PluginNotePortsImpl {
    fn count(&self, is_input: bool) -> u32;
    fn get(&self, is_input: bool, index: u32, writer: &mut NotePortInfoWriter);
}

impl<P: Plugin> ExtensionImplementation<P> for PluginNotePorts
where
    for<'a> P::MainThread<'a>: PluginNotePortsImpl,
{
    const IMPLEMENTATION: &'static Self = &PluginNotePorts(
        clap_plugin_note_ports {
            count: Some(count::<P>),
            get: Some(get::<P>),
        },
        PhantomData,
    );
}

unsafe extern "C" fn count<P: Plugin>(plugin: *const clap_plugin, is_input: bool) -> u32
where
    for<'a> P::MainThread<'a>: PluginNotePortsImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| Ok(p.main_thread().as_ref().count(is_input)))
        .unwrap_or(0)
}

unsafe extern "C" fn get<P: Plugin>(
    plugin: *const clap_plugin,
    index: u32,
    is_input: bool,
    info: *mut clap_note_port_info,
) -> bool
where
    for<'a> P::MainThread<'a>: PluginNotePortsImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| {
        if info.is_null() {
            return Err(PluginWrapperError::NulPtr("clap_note_port_info"));
        };

        let mut writer = NotePortInfoWriter::from_raw(info);
        p.main_thread().as_ref().get(is_input, index, &mut writer);
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
