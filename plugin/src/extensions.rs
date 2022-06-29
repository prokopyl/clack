//! Traits and associated utilities to handle and implement CLAP extensions.
//!
//! These traits are designed to be used for *implementing* custom or unsupported extensions.
//! If you want to use an existing extension in your plugin, see the `clack_extensions`
//! crate instead.
//!
//! # Example
//!
//! This example shows a basic implementation for the plugin side of the CLAP State extension.
//!
//! The implementation wrapper leverages the [`PluginWrapper`](crate::plugin::wrapper::PluginWrapper)
//! utility to handle things like error management and unwind safety. See its documentation for more
//! information.
//!
//! ```
//! # use std::os::raw::c_char;
//! use clap_sys::ext::state::{CLAP_EXT_STATE, clap_plugin_state};
//! use clack_common::extensions::{Extension, ExtensionImplementation, PluginExtension};
//!
//! // The struct end-users will actually interact with.
//! #[repr(C)]
//! pub struct PluginState(clap_plugin_state);
//!
//! unsafe impl Extension for PluginState {
//!     const IDENTIFIER: *const c_char = CLAP_EXT_STATE;
//!     type ExtensionType = PluginExtension;
//! }
//!
//! // For implementors of the extensions (here, on the plugin side):
//! // first define a trait the extension has to implement
//! use clack_common::stream::{InputStream, OutputStream};
//! use clack_plugin::plugin::PluginError;
//!
//! pub trait PluginStateImplementation {
//!     fn load(&mut self, input: &mut InputStream) -> Result<(), PluginError>;
//! }
//!
//! // Then, implement the ExtensionImplementation trait for the given implementors
//! // to provide the C FFI-compatible struct.
//! use clack_plugin::plugin::Plugin;
//!
//! impl<'a, P: Plugin<'a>> ExtensionImplementation<P> for PluginState
//! where
//!     // In this case, all of the CLAP State methods belong to the main thread.
//!     // Other extensions may have other requirements, possibly split between multiple threads.
//!     P::MainThread: PluginStateImplementation,
//! {
//!     const IMPLEMENTATION: &'static Self = &PluginState(clap_plugin_state {
//!         # save: save,
//!         // For the sake of this example, we are only implementing the load() method.
//!         load: load::<P>,
//!     });
//! }
//! # unsafe extern "C" fn save(_: *const clap_plugin, _: *const clap_sys::stream::clap_ostream) -> bool {
//! #    unimplemented!()
//! # }
//!
//! // Finally, implement the C FFI functions that will be exposed to the host.
//! use clap_sys::plugin::clap_plugin;
//! use clap_sys::stream::clap_istream;
//! use clack_plugin::plugin::wrapper::PluginWrapper;
//!
//! unsafe extern "C" fn load<'a, P: Plugin<'a>>(
//!     plugin: *const clap_plugin,
//!     stream: *const clap_istream,
//! ) -> bool
//! where
//!     P::MainThread: PluginStateImplementation,
//! {
//!     PluginWrapper::<P>::handle(plugin, |p| {
//!         let input = InputStream::from_raw_mut(&mut *(stream as *mut _));
//!         // Retrieve the plugin's main thread struct, and call load() on it
//!         p.main_thread().as_mut().load(input)?;
//!         Ok(())
//!     })
//!     .is_some()
//! }
//! ```
//!
//!

use crate::plugin::Plugin;
use core::ffi::c_void;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub use clack_common::extensions::*;

/// A collection of all extensions supported for a given plugin type `P`.
///
/// Plugins can declare the different extensions they support by using the
/// [`register`](PluginExtensions::register) method on this struct, during a call to
/// [`declare_extensions`](crate::plugin::Plugin::declare_extensions).
pub struct PluginExtensions<'a, P> {
    found: Option<NonNull<c_void>>,
    requested: &'a CStr,
    plugin_type: PhantomData<P>,
}

impl<'a, 'b, P: Plugin<'b>> PluginExtensions<'a, P> {
    #[inline]
    pub(crate) fn new(requested: &'a CStr) -> Self {
        Self {
            found: None,
            requested,
            plugin_type: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn found(&self) -> *const c_void {
        self.found
            .map(|p| p.as_ptr())
            .unwrap_or(::core::ptr::null_mut())
    }

    /// Adds a given extension implementation to the list of extensions this plugin supports.
    pub fn register<E: ExtensionImplementation<P, ExtensionType = PluginExtension>>(
        &mut self,
    ) -> &mut Self {
        if self.found.is_some() {
            return self;
        }

        let uri = unsafe { CStr::from_ptr(E::IDENTIFIER) };
        if uri == self.requested {
            self.found = NonNull::new(E::IMPLEMENTATION as *const _ as *mut _)
        }

        self
    }
}
