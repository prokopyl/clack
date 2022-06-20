use clack_common::extensions::{Extension, HostExtension, PluginExtension};
use clack_common::stream::{InputStream, OutputStream};
use clack_plugin::plugin::PluginError;
use clap_sys::ext::state::{clap_host_state, clap_plugin_state, CLAP_EXT_STATE};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::os::raw::c_char;

#[repr(C)]
pub struct PluginState(clap_plugin_state, PhantomData<*const clap_plugin_state>);

unsafe impl Extension for PluginState {
    const IDENTIFIER: *const c_char = CLAP_EXT_STATE;
    type ExtensionType = PluginExtension;
}

#[repr(C)]
pub struct HostState(clap_host_state, PhantomData<*const clap_host_state>);

unsafe impl Extension for HostState {
    const IDENTIFIER: *const c_char = CLAP_EXT_STATE;
    type ExtensionType = HostExtension;
}

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
    use clack_host::plugin::PluginMainThread;
    use std::io::{Read, Write};

    impl PluginState {
        pub fn load<R: Read>(
            &self,
            plugin: PluginMainThread,
            reader: &mut R,
        ) -> Result<(), StateError> {
            let mut stream = InputStream::from_reader(reader);

            let success = if let Some(load) = self.0.load {
                unsafe { load(plugin.as_raw(), stream.as_raw_mut()) }
            } else {
                false
            };

            match success {
                true => Ok(()),
                false => Err(StateError { saving: false }),
            }
        }

        pub fn save<W: Write>(
            &self,
            plugin: PluginMainThread,
            writer: &mut W,
        ) -> Result<(), StateError> {
            let mut stream = OutputStream::from_writer(writer);

            let result = if let Some(save) = self.0.save {
                unsafe { save(plugin.as_raw(), stream.as_raw_mut()) }
            } else {
                false
            };

            match result {
                true => Ok(()),
                false => Err(StateError { saving: true }),
            }
        }
    }
}
