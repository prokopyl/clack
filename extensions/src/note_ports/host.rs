use super::*;
use clack_common::extensions::ExtensionImplementation;
use clack_host::host::Host;
use clack_host::plugin::PluginMainThreadHandle;
use clack_host::wrapper::HostWrapper;
use clap_sys::host::clap_host;

impl PluginNotePorts {
    pub fn count(&self, plugin: &PluginMainThreadHandle, is_input: bool) -> u32 {
        match self.0.count {
            None => 0,
            Some(count) => unsafe { count(plugin.as_raw(), is_input) },
        }
    }

    pub fn get<'b>(
        &self,
        plugin: &PluginMainThreadHandle,
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

pub trait HostNotePortsImplementation {
    fn supported_dialects(&self) -> NoteDialects;
    fn rescan(&self, flags: NotePortRescanFlags);
}

impl<H: for<'h> Host<'h>> ExtensionImplementation<H> for HostNotePorts
where
    for<'h> <H as Host<'h>>::MainThread: HostNotePortsImplementation,
{
    const IMPLEMENTATION: &'static Self = &HostNotePorts(
        clap_host_note_ports {
            supported_dialects: Some(supported_dialects::<H>),
            rescan: Some(rescan::<H>),
        },
        PhantomData,
    );
}

unsafe extern "C" fn supported_dialects<H: for<'a> Host<'a>>(host: *const clap_host) -> u32
where
    for<'a> <H as Host<'a>>::MainThread: HostNotePortsImplementation,
{
    HostWrapper::<H>::handle(host, |host| {
        Ok(host.main_thread().as_ref().supported_dialects().bits)
    })
    .unwrap_or(0)
}

unsafe extern "C" fn rescan<H: for<'a> Host<'a>>(host: *const clap_host, flag: u32)
where
    for<'a> <H as Host<'a>>::MainThread: HostNotePortsImplementation,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread()
            .as_mut()
            .rescan(NotePortRescanFlags::from_bits_truncate(flag));

        Ok(())
    });
}
