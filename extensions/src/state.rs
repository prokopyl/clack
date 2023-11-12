//! Allows plugins to save and restore state using host-managed raw binary storage streams.
//!
//! This extension is to be used as the backing storage for both parameter values and any other
//! non-parameter state.
//!
//! Clack uses the [`InputStream`](clack_common::stream::InputStream) and
//! [`OutputStream`](clack_common::stream::OutputStream)
//!
//! Plugins can also notify the host that their state has changed compared to the last time it was
//! saved or loaded, using the `mark_dirty` call.
//!
//! # Host-Side Example
//!
//! ```
//! use std::error::Error;
//! use std::io::Cursor;
//! use clack_extensions::state::{HostState, HostStateImpl, PluginState};
//! use clack_host::prelude::*;
//!
//! struct MyHost;
//!
//! impl Host for MyHost {
//!     type Shared<'a> = MyHostShared<'a>;
//!     type MainThread<'a> = MyHostMainThread<'a>;
//!     type AudioProcessor<'a> = ();
//!
//!     fn declare_extensions(builder: &mut HostExtensions<Self>, _: &Self::Shared<'_>) {
//!         builder.register::<HostState>();
//!     }
//! }
//!
//! struct MyHostShared<'a> {
//!     state_ext: Option<&'a PluginState>
//! }
//!
//! impl<'a> HostShared<'a> for MyHostShared<'a> {
//!     fn instantiated(&mut self, instance: PluginSharedHandle<'a>) {
//!         self.state_ext = instance.get_extension()
//!     }
//!     # fn request_restart(&self) { unimplemented!() }
//!     # fn request_process(&self) { unimplemented!() }
//!     # fn request_callback(&self) { unimplemented!() }
//! }
//!
//! struct MyHostMainThread<'a> {
//!     shared: &'a MyHostShared<'a>,
//!     plugin: Option<PluginMainThreadHandle<'a>>,
//!     is_state_dirty: bool
//! }
//!
//! impl<'a> HostMainThread<'a> for MyHostMainThread<'a> {
//!     /* ... */
//! #    fn instantiated(&mut self, instance: PluginMainThreadHandle<'a>) { self.plugin = Some(instance); }
//! }
//!
//! // Implement the Host State extension for the plugin to notify us of its dirty save state
//! impl<'a> HostStateImpl for MyHostMainThread<'a> {
//!     fn mark_dirty(&mut self) {
//!         // Notify the user that the plugin should now be saved.
//!         // For this example, we'll just use a boolean.
//!         self.is_state_dirty = true;
//!     }
//! }
//!
//! // Implement our helper functions for loading and saving state.
//!
//! impl<'a> MyHostMainThread<'a> {
//!     /// This loads the plugin's state from the given raw byte array
//!     pub fn load_state(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
//!         let plugin = self.plugin.as_mut()
//!             .expect("Plugin is not yet instantiated");
//!         let state_ext = self.shared.state_ext
//!             .expect("Plugin does not implement State extension");
//!
//!         let mut reader = Cursor::new(data);
//!         state_ext.load(plugin, &mut reader)?;
//!
//!         Ok(())
//!     }
//!
//!     /// Exports the current plugin state into a raw byte array (Vec) to be reloaded later.
//!     pub fn save_state(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
//!         let plugin = self.plugin.as_mut()
//!             .expect("Plugin is not yet instantiated");
//!         let state_ext = self.shared.state_ext
//!             .expect("Plugin does not implement State extension");
//!
//!         let mut buffer = Vec::new();
//!         state_ext.save(plugin, &mut buffer)?;
//!  
//!         Ok(buffer)
//!     }
//! }
//! # pub fn main() -> Result<(), Box<dyn Error>> {
//! # mod utils { include!("./__doc_utils.rs"); }
//! let mut plugin_instance: PluginInstance<MyHost> = /* ... */
//! # utils::get_working_instance(|_| MyHostShared { state_ext: None }, |shared| MyHostMainThread { is_state_dirty: false, shared, plugin: None })?;
//!
//! // We just loaded our plugin, but we have a preset to initialize it to.
//! let preset_data = b"I'm a totally legit preset.";
//! plugin_instance.main_thread_host_data_mut().load_state(preset_data)?;
//!
//! // Some time passes, user interacts with the plugin, etc.
//! // Now the user wants to save the state.
//! let saved_state: Vec<u8> = plugin_instance.main_thread_host_data_mut().save_state()?;
//! # Ok(()) }
//! ```

use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide};
use clap_sys::ext::state::{clap_host_state, clap_plugin_state, CLAP_EXT_STATE};
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[repr(C)]
pub struct PluginState(clap_plugin_state, PhantomData<*const clap_plugin_state>);

unsafe impl Extension for PluginState {
    const IDENTIFIER: &'static CStr = CLAP_EXT_STATE;
    type ExtensionSide = PluginExtensionSide;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginState {}
unsafe impl Sync for PluginState {}

#[repr(C)]
pub struct HostState(clap_host_state, PhantomData<*const clap_host_state>);

unsafe impl Extension for HostState {
    const IDENTIFIER: &'static CStr = CLAP_EXT_STATE;
    type ExtensionSide = HostExtensionSide;
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
mod host;
#[cfg(feature = "clack-host")]
pub use host::*;
