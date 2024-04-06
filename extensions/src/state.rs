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
//! use std::sync::OnceLock;
//! use clack_extensions::state::{HostState, HostStateImpl, PluginState};
//! use clack_host::prelude::*;
//!
//! struct MyHost;
//!
//! impl HostHandlers for MyHost {
//!     type Shared<'a> = MyHostShared;
//!     type MainThread<'a> = MyHostMainThread<'a>;
//!     type AudioProcessor<'a> = ();
//!
//!     fn declare_extensions(builder: &mut HostExtensions<Self>, _: &Self::Shared<'_>) {
//!         builder.register::<HostState>();
//!     }
//! }
//!
//! struct MyHostShared {
//!     state_ext: OnceLock<Option<PluginState>>
//! }
//!
//! impl<'a> SharedHandler<'a> for MyHostShared {
//!     fn initializing(&self, instance: InitializingPluginHandle<'a>) {
//!         let _ = self.state_ext.set(instance.get_extension());
//!     }
//!     # fn request_restart(&self) { unimplemented!() }
//!     # fn request_process(&self) { unimplemented!() }
//!     # fn request_callback(&self) { unimplemented!() }
//! }
//!
//! struct MyHostMainThread<'a> {
//!     shared: &'a MyHostShared,
//!     is_state_dirty: bool
//! }
//!
//! impl<'a> MainThreadHandler<'a> for MyHostMainThread<'a> {
//!     /* ... */
//! #    fn initialized(&mut self, _instance: InitializedPluginHandle<'a>) {}
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
//! # pub fn main() -> Result<(), Box<dyn Error>> {
//! # mod utils { include!("./__doc_utils.rs"); }
//! let mut plugin_instance: PluginInstance<MyHost> = /* ... */
//! # utils::get_working_instance(|_| MyHostShared { state_ext: OnceLock::new() }, |shared| MyHostMainThread { is_state_dirty: false, shared })?;
//!
//! let state_ext = plugin_instance.shared_handler().state_ext
//!     .get()
//!     .expect("Plugin is not yet instantiated")
//!     .expect("Plugin does not implement State extension");
//!
//! // We just loaded our plugin, but we have a preset to initialize it to.
//! let preset_data = b"I'm a totally legit preset.";
//! let mut reader = Cursor::new(preset_data);
//! state_ext.load(&mut plugin_instance.plugin_handle(), &mut reader)?;
//!
//! // Some time passes, user interacts with the plugin, etc.
//! // Now the user wants to save the state.
//! let mut buffer = Vec::new();
//! state_ext.save(&mut plugin_instance.plugin_handle(), &mut buffer)?;
//! # Ok(()) }
//! ```

use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::state::{clap_host_state, clap_plugin_state, CLAP_EXT_STATE};
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginState(RawExtension<PluginExtensionSide, clap_plugin_state>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginState {
    const IDENTIFIER: &'static CStr = CLAP_EXT_STATE;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostState(RawExtension<HostExtensionSide, clap_host_state>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostState {
    const IDENTIFIER: &'static CStr = CLAP_EXT_STATE;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StateError {
    saving: bool,
}

impl StateError {
    /// Returns a [`StateError`] that was triggered while loading state.
    ///
    /// This information is used in the error's message.
    pub const fn loading() -> Self {
        Self { saving: false }
    }

    /// Returns a [`StateError`] that was triggered while saving state.
    ///
    /// This information is used in the error's message.
    pub const fn saving() -> Self {
        Self { saving: true }
    }
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
