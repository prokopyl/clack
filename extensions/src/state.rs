use clack_common::extensions::{Extension, HostExtension, PluginExtension};
use clap_sys::ext::state::{clap_host_state, clap_plugin_state, CLAP_EXT_STATE};
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[repr(C)]
pub struct PluginState(clap_plugin_state, PhantomData<*const clap_plugin_state>);

unsafe impl Extension for PluginState {
    const IDENTIFIER: &'static CStr = CLAP_EXT_STATE;
    type ExtensionType = PluginExtension;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginState {}
unsafe impl Sync for PluginState {}

#[repr(C)]
pub struct HostState(clap_host_state, PhantomData<*const clap_host_state>);

unsafe impl Extension for HostState {
    const IDENTIFIER: &'static CStr = CLAP_EXT_STATE;
    type ExtensionType = HostExtension;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostState {}
unsafe impl Sync for HostState {}

#[derive(Copy, Clone, Debug)]
pub struct StateError {
    saving: bool,
}

impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.saving {
            f.write_str("Failed to save plugin state")
        } else {
            f.write_str("Failed to load plugin state")
        }
    }
}

impl Error for StateError {}

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_common::stream::{InputStream, OutputStream};
    use clack_host::plugin::PluginMainThreadHandle;
    use std::io::{Read, Write};

    impl PluginState {
        pub fn load<R: Read>(
            &self,
            plugin: PluginMainThreadHandle,
            reader: &mut R,
        ) -> Result<(), StateError> {
            let mut stream = InputStream::from_reader(reader);

            if unsafe {
                (self.0.load.ok_or(StateError { saving: false })?)(
                    plugin.as_raw(),
                    stream.as_raw_mut(),
                )
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
                (self.0.save.ok_or(StateError { saving: true })?)(
                    plugin.as_raw(),
                    stream.as_raw_mut(),
                )
            } {
                Ok(())
            } else {
                Err(StateError { saving: true })
            }
        }
    }
}
