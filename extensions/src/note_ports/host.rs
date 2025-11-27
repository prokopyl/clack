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
            inner: MaybeUninit::zeroed(),
        }
    }
}

impl PluginNotePorts {
    pub fn count(&self, plugin: &mut PluginMainThreadHandle, is_input: bool) -> u32 {
        match plugin.use_extension(&self.0).count {
            None => 0,
            // SAFETY: This type ensures the function pointer is valid.
            Some(count) => unsafe { count(plugin.as_raw(), is_input) },
        }
    }

    pub fn get<'b>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        index: u32,
        is_input: bool,
        buffer: &'b mut NotePortInfoBuffer,
    ) -> Option<NotePortInfo<'b>> {
        let success =
            // SAFETY: This type ensures the function pointer is valid.
            unsafe { plugin.use_extension(&self.0).get?(plugin.as_raw(), index, is_input, buffer.inner.as_mut_ptr()) };

        if success {
            // SAFETY: we just checked the buffer was successfully written to
            Some(unsafe { NotePortInfo::from_raw(buffer.inner.assume_init_ref())? })
        } else {
            None
        }
    }
}

pub trait HostNotePortsImpl {
    fn supported_dialects(&self) -> NoteDialects;
    fn rescan(&mut self, flags: NotePortRescanFlags);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H: HostHandlers> ExtensionImplementation<H> for HostNotePorts
where
    for<'h> <H as HostHandlers>::MainThread<'h>: HostNotePortsImpl,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&&clap_host_note_ports {
            supported_dialects: Some(supported_dialects::<H>),
            rescan: Some(rescan::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn supported_dialects<H: HostHandlers>(host: *const clap_host) -> u32
where
    for<'a> <H as HostHandlers>::MainThread<'a>: HostNotePortsImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        Ok(host.main_thread().as_ref().supported_dialects().bits())
    })
    .unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn rescan<H: HostHandlers>(host: *const clap_host, flag: u32)
where
    for<'a> <H as HostHandlers>::MainThread<'a>: HostNotePortsImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(NotePortRescanFlags::from_bits_truncate(flag));

        Ok(())
    });
}
