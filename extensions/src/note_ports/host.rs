use super::*;
use clack_host::extensions::prelude::*;
use std::mem::MaybeUninit;

#[derive(Clone)]
pub struct NotePortInfoBuffer {
    inner: MaybeUninit<clap_note_port_info>,
}

impl Default for NotePortInfoBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl NotePortInfoBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::uninit(),
        }
    }
}

impl PluginNotePorts {
    pub fn count(&self, plugin: &mut PluginMainThreadHandle, is_input: bool) -> u32 {
        match self.0.count {
            None => 0,
            Some(count) => unsafe { count(plugin.as_raw(), is_input) },
        }
    }

    pub fn get<'b>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        index: u32,
        is_input: bool,
        buffer: &'b mut NotePortInfoBuffer,
    ) -> Option<NotePortInfoData<'b>> {
        let success =
            unsafe { (self.0.get?)(plugin.as_raw(), index, is_input, buffer.inner.as_mut_ptr()) };

        if success {
            unsafe { NotePortInfoData::try_from_raw(buffer.inner.assume_init_ref()) }
        } else {
            None
        }
    }
}

pub trait HostNotePortsImpl {
    fn supported_dialects(&self) -> NoteDialects;
    fn rescan(&mut self, flags: NotePortRescanFlags);
}

impl<H: Host> ExtensionImplementation<H> for HostNotePorts
where
    for<'h> <H as Host>::MainThread<'h>: HostNotePortsImpl,
{
    const IMPLEMENTATION: &'static Self = &HostNotePorts(
        clap_host_note_ports {
            supported_dialects: Some(supported_dialects::<H>),
            rescan: Some(rescan::<H>),
        },
        PhantomData,
    );
}

unsafe extern "C" fn supported_dialects<H: Host>(host: *const clap_host) -> u32
where
    for<'a> <H as Host>::MainThread<'a>: HostNotePortsImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        Ok(host.main_thread().as_ref().supported_dialects().bits)
    })
    .unwrap_or(0)
}

unsafe extern "C" fn rescan<H: Host>(host: *const clap_host, flag: u32)
where
    for<'a> <H as Host>::MainThread<'a>: HostNotePortsImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(NotePortRescanFlags::from_bits_truncate(flag));

        Ok(())
    });
}
