use super::*;
use clack_common::stream::{InputStream, OutputStream};
use clack_host::extensions::prelude::*;
use std::io::{Read, Write};

impl PluginState {
    pub fn load<R: Read>(
        &self,
        plugin: PluginMainThreadHandle,
        reader: &mut R,
    ) -> Result<(), StateError> {
        let mut stream = InputStream::from_reader(reader);

        if unsafe {
            (self.0.load.ok_or(StateError { saving: false })?)(plugin.as_raw(), stream.as_raw_mut())
        } {
            Ok(())
        } else {
            Err(StateError { saving: false })
        }
    }

    pub fn save<W: Write>(
        &self,
        plugin: PluginMainThreadHandle,
        writer: &mut W,
    ) -> Result<(), StateError> {
        let mut stream = OutputStream::from_writer(writer);

        if unsafe {
            (self.0.save.ok_or(StateError { saving: true })?)(plugin.as_raw(), stream.as_raw_mut())
        } {
            Ok(())
        } else {
            Err(StateError { saving: true })
        }
    }
}

pub trait HostStateImpl {
    fn mark_dirty(&mut self);
}

impl<H: Host> ExtensionImplementation<H> for HostState
where
    for<'a> <H as Host>::MainThread<'a>: HostStateImpl,
{
    #[doc(hidden)]
    const IMPLEMENTATION: &'static Self = &Self(
        clap_host_state {
            mark_dirty: Some(mark_dirty::<H>),
        },
        PhantomData,
    );
}

unsafe extern "C" fn mark_dirty<H: Host>(host: *const clap_host)
where
    for<'a> <H as Host>::MainThread<'a>: HostStateImpl,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread().as_mut().mark_dirty();

        Ok(())
    });
}
