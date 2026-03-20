use super::*;
use clack_common::stream::{InputStream, OutputStream};
use clack_host::extensions::prelude::*;
use std::io::{Read, Write};

impl PluginState {
    /// Loads the plugin state from a given byte stream.
    ///
    /// The byte stream can come from any object implementing [`Read`].
    ///
    /// # Errors
    ///
    /// If this operation fails, a [`StateError`] is returned.
    pub fn load<R: Read>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        reader: &mut R,
    ) -> Result<(), StateError> {
        let mut stream = InputStream::from_reader(reader);

        // SAFETY: This type ensures the function pointer is valid.
        if unsafe {
            (plugin
                .use_extension(&self.0)
                .load
                .ok_or(StateError { saving: false })?)(
                plugin.as_raw(), stream.as_raw_mut()
            )
        } {
            Ok(())
        } else {
            Err(StateError { saving: false })
        }
    }

    /// Saves the plugin state into a given byte stream.
    ///
    /// The byte stream can be any object implementing [`Write`].
    ///
    /// # Errors
    ///
    /// If this operation fails, a [`StateError`] is returned.
    pub fn save<W: Write>(
        &self,
        plugin: &mut PluginMainThreadHandle,
        writer: &mut W,
    ) -> Result<(), StateError> {
        let mut stream = OutputStream::from_writer(writer);

        // SAFETY: This type ensures the function pointer is valid.
        if unsafe {
            (plugin
                .use_extension(&self.0)
                .save
                .ok_or(StateError { saving: true })?)(
                plugin.as_raw(), stream.as_raw_mut()
            )
        } {
            Ok(())
        } else {
            Err(StateError { saving: true })
        }
    }
}

/// Implementation of the Host side of the State extension.
pub trait HostStateImpl {
    /// Tells the host that the plugin state has changed, and may need to be saved again.
    ///
    /// Note that if a parameter value changes, it is implicit that the state is dirty.
    fn mark_dirty(&mut self);
}

// SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
unsafe impl<H: HostHandlers> ExtensionImplementation<H> for HostState
where
    for<'a> H: HostHandlers<MainThread<'a>: HostStateImpl>,
{
    #[doc(hidden)]
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_state {
            mark_dirty: Some(mark_dirty::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn mark_dirty<H>(host: *const clap_host)
where
    for<'a> H: HostHandlers<MainThread<'a>: HostStateImpl>,
{
    HostWrapper::<H>::handle(host, |host| {
        host.main_thread().as_mut().mark_dirty();

        Ok(())
    });
}
