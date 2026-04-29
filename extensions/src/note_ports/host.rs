use super::*;
use clack_host::extensions::prelude::*;
use std::mem::MaybeUninit;

/// A scratch buffer for the plugin to write note port metadata to.
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
    /// Get an empty buffer for the plugin to write note port metadata into.
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: MaybeUninit::zeroed(),
        }
    }
}

impl PluginNotePorts {
    /// Returns number of note ports, for either input or output.
    pub fn count(&self, plugin: &mut PluginMainThreadHandle, is_input: bool) -> u32 {
        match plugin.use_extension(&self.0).count {
            None => 0,
            // SAFETY: This type ensures the function pointer is valid.
            Some(count) => unsafe { count(plugin.as_raw(), is_input) },
        }
    }

    /// Get information about a note port by its index, for either input or output.
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

/// Implementation of the Host-side of the Note Ports extension.
pub trait HostNotePortsImpl {
    /// Query which note dialects are supported by the host.
    fn supported_dialects(&self) -> NoteDialects;

    /// Rescan the full list of note ports according to the flags.
    /// See [`NotePortRescanFlags`] for more details.
    fn rescan(&mut self, flags: NotePortRescanFlags);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H> ExtensionImplementation<H> for HostNotePorts
where
    for<'h> H: HostHandlers<MainThread<'h>: HostNotePortsImpl>,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&&clap_host_note_ports {
            supported_dialects: Some(supported_dialects::<H>),
            rescan: Some(rescan::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn supported_dialects<H>(host: *const clap_host) -> u32
where
    for<'h> H: HostHandlers<MainThread<'h>: HostNotePortsImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        Ok(host.main_thread().as_ref().supported_dialects().bits())
    })
    .unwrap_or(0)
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn rescan<H>(host: *const clap_host, flags: u32)
where
    for<'h> H: HostHandlers<MainThread<'h>: HostNotePortsImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(NotePortRescanFlags::from_bits_truncate(flags));

        Ok(())
    });
}
