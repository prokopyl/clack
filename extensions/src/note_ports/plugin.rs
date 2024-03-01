use super::*;
use crate::utils::write_to_array_buf;
use clack_plugin::extensions::prelude::*;
use std::mem::MaybeUninit;
use std::ptr::addr_of_mut;

pub struct NotePortInfoWriter<'a> {
    buf: &'a mut MaybeUninit<clap_note_port_info>,
    is_set: bool,
}

impl<'a> NotePortInfoWriter<'a> {
    /// # Safety
    ///
    /// The user must ensure the provided pointer is aligned and points to a valid allocation.
    /// However, it doesn't have to be initialized.
    #[inline]
    unsafe fn from_raw(raw: *mut clap_note_port_info) -> Self {
        Self {
            buf: &mut *raw.cast(),
            is_set: false,
        }
    }

    #[inline]
    pub fn set(&mut self, info: &NotePortInfo) {
        use core::ptr::write;

        let buf = self.buf.as_mut_ptr();

        // SAFETY: all pointers come from `buf`, which is valid for writes and well-aligned
        unsafe {
            write(addr_of_mut!((*buf).id), info.id);
            write_to_array_buf(addr_of_mut!((*buf).name), info.name);

            write(
                addr_of_mut!((*buf).supported_dialects),
                info.supported_dialects.bits(),
            );
            write(
                addr_of_mut!((*buf).preferred_dialect),
                info.preferred_dialect.map(|d| d as u32).unwrap_or(0),
            );
        }

        self.is_set = true;
    }
}

pub trait PluginNotePortsImpl {
    fn count(&mut self, is_input: bool) -> u32;
    fn get(&mut self, index: u32, is_input: bool, writer: &mut NotePortInfoWriter);
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

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn count<P: Plugin>(plugin: *const clap_plugin, is_input: bool) -> u32
where
    for<'a> P::MainThread<'a>: PluginNotePortsImpl,
{
    PluginWrapper::<P>::handle(plugin, |p| Ok(p.main_thread().as_mut().count(is_input)))
        .unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
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
        p.main_thread().as_mut().get(index, is_input, &mut writer);
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
                // SAFETY: This type ensures the function pointer is valid.
                NoteDialects::from_bits_truncate(unsafe { supported(host.as_raw()) })
            }
        }
    }

    #[inline]
    pub fn rescan(&self, host: &mut HostMainThreadHandle, flags: NotePortRescanFlags) {
        if let Some(rescan) = self.0.rescan {
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { rescan(host.as_raw(), flags.bits()) }
        }
    }
}
