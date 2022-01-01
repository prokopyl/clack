use clack_common::extensions::{Extension, HostExtension, PluginExtension};
use clack_common::stream::{InputStream, OutputStream};
use clack_plugin::plugin::PluginError;
use clap_sys::ext::state::{clap_host_state, clap_plugin_state, CLAP_EXT_STATE};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[repr(C)]
pub struct PluginState(clap_plugin_state, PhantomData<*const clap_plugin_state>);

unsafe impl<'a> Extension<'a> for PluginState {
    const IDENTIFIER: *const u8 = CLAP_EXT_STATE as *const _;
    type ExtensionType = PluginExtension;
}

#[repr(C)]
pub struct HostState(clap_host_state, PhantomData<*const clap_host_state>);

unsafe impl<'a> Extension<'a> for HostState {
    const IDENTIFIER: *const u8 = CLAP_EXT_STATE as *const _;
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

#[cfg(feature = "clack-plugin")]
mod plugin;
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_common::stream::{InputStream, OutputStream};
    use clack_host::instance::PluginInstance;
    use std::io::{Read, Write};

    impl PluginState {
        pub fn load<R: Read>(
            &self,
            plugin: &mut PluginInstance,
            reader: &mut R,
        ) -> Result<(), StateError> {
            let mut stream = InputStream::from_reader(reader);
            let result = unsafe { (self.0.load)(plugin.as_raw(), stream.as_raw_mut()) };
            match result {
                true => Ok(()),
                false => Err(StateError { saving: false }),
            }
        }

        pub fn save<W: Write>(
            &self,
            plugin: &mut PluginInstance,
            writer: &mut W,
        ) -> Result<(), StateError> {
            let mut stream = OutputStream::from_writer(writer);
            let result = unsafe { (self.0.save)(plugin.as_raw(), stream.as_raw_mut()) };
            match result {
                true => Ok(()),
                false => Err(StateError { saving: true }),
            }
        }
    }
}
