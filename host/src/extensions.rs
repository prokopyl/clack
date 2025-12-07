//! The host side of the Clack extension system.
//!
//! The majority of the features in CLAP come from extensions, including
//! parameters management, state loading and saving, GUI handling, and more.
//!
//! The goal of this system is to maximize flexibility and extensibility, while also preventing
//! feature creep by not requiring all hosts to implement every single extension. This module
//! provides thin wrappers that maintain those abilities while also enforcing both type safety and
//! memory safety.
//!
//! This crate does not include any extension itself. All official, first-party CLAP extensions are
//! implemented on top of it in the `clack-extensions` crate, but all extension implementations are
//! treated as first-class citizens by both CLAP and Clack, regardless of their provenance. See the
//! [Creating custom extensions](#creating-custom-extensions) section below for more information on
//! how to implement third-party extensions.
//!
//! At instantiation time, both the plugin and the host will query each other's declared supported
//! extensions. When one side declares supporting an extension, it will also provide an [`Extension`]
//! object containing that extension's ABI to the other side. See the [`Extension`] type
//! documentation for more information on how to use and store them.
//!
//! [`Extension`] ABIs are split in two parts: one that is exposed by the host, and one that is
//! exposed by the plugin. For instance, for the `Latency` extension, the ABIs are named `HostLatency`
//! and `PluginLatency` respectively.
//!
//! # Using extensions in a Clack host
//!
//! Supporting a specific extension in a CLAP host has two requirements:
//!
//! * Querying a plugin for its side of the ABI, and consuming it.
//!   
//!   This is the most straightforward part: once the plugin is instantiated and the host can access
//!   its [`PluginSharedHandle`](crate::plugin::PluginSharedHandle), it can use the
//!   [`PluginSharedHandle::get_extension`](crate::plugin::PluginSharedHandle::get_extension)
//!   method to query the plugin for any supported extension, and store its associated ABI.
//!
//!   References to an Extension ABI can be shared, copied and used in any thread as long as they
//!   don't outlive the plugin instance. They are therefore most commonly stored in the host's
//!   [`HostShared`](crate::host::SharedHandler) associated type, as shown in the example below.
//!
//!
//! * Implementing the host side of the ABI, and exposing it to the plugin to be queried.
//!
//!   All extensions in Clack have at least one trait to be implemented onto a specific
//!   [`Host`](crate::host::HostHandlers) subtype ([`HostMainThread`](crate::host::MainThreadHandler),
//!   [`HostAudioProcessor`](crate::host::AudioProcessorHandler),
//!   or [`HostShared`](crate::host::SharedHandler)), depending on the thread specification of the
//!   ABI's method. For example, the `Log` extension's ABI has to be fully thread-safe, therefore
//!   the `HostLogImpl` trait has to be implemented on the [`HostShared`](crate::host::SharedHandler)
//!   type.
//!
//!   See the [`host`](crate::host) module documentation to know more about CLAP's
//!   thread specification.
//!   
//!   Sometimes however, some ABIs expose different methods in different thread classes, leading
//!   to that many traits to be implemented on different types. For instance, the `Params` ABI
//!   exposes one thread-safe method and two that are main-thread only. Therefore, the
//!   `HostParamsImplShared` and `HostParamsImplMainThread` traits have to be implemented on the
//!   [`HostShared`](crate::host::SharedHandler) and [`HostMainThread`](crate::host::MainThreadHandler)
//!   types, respectively.
//!
//!   Once this is all done, the host implementation can declare this extension by using the
//!   [`HostExtensions::register`](crate::host::HostExtensions::register) method in the
//!   [`Host::declare_extensions`](crate::host::HostHandlers::declare_extensions) method implementation.
//!
//!   The fact that the right traits are implemented on the right [`Host`](crate::host::HostHandlers)
//!   associated types is automatically checked at compile time, upon calling the
//!   [`HostExtensions::register`](crate::host::HostExtensions::register) method.
//!
//! ## Example
//!
//! This example implements a host supporting the `Latency` extension.
//!
//! ```
//! use std::sync::OnceLock;
//! use clack_host::prelude::*;
//! use clack_extensions::latency::*;
//!
//! #[derive(Default)]
//! struct MyHostShared {
//!     // Queried extension
//!     // Note this may be None even after instantiation,
//!     // in case the extension isn't supported by the plugin.
//!     latency_extension: OnceLock<Option<PluginLatency>>
//! }
//!
//! impl<'a> SharedHandler<'a> for MyHostShared {
//!     // We can query the plugin's extensions as soon as the plugin starts initializing
//!     fn initializing(&self, instance: InitializingPluginHandle<'a>) {
//!         let _ = self.latency_extension.set(instance.get_extension());
//!     }
//!     
//!     /* ... */
//!     # fn request_restart(&self) { unimplemented!() }
//!     # fn request_process(&self) { unimplemented!() }
//!     # fn request_callback(&self) { unimplemented!() }
//! }
//!
//! struct MyHostMainThread<'a> {
//!     shared: &'a MyHostShared,
//!     instance: Option<InitializedPluginHandle<'a>>,
//!
//!     // The latency that is sent to us by the plugin's Latency extension.
//!     latency_changed: bool
//! }
//!
//! impl<'a> MainThreadHandler<'a> for MyHostMainThread<'a> {
//!     // The plugin's instance handle is required to call extension methods.
//!     fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
//!         self.instance = Some(instance);
//!     }
//! }
//!
//! impl<'a> HostLatencyImpl for MyHostMainThread<'a> {
//!     // This method is called by the plugin whenever its latency changed.
//!     fn changed(&mut self) {
//!         // Ensure that the plugin is instantiated and supports the Latency extension.
//!         if let Some(Some(_latency)) = self.shared.latency_extension.get() {
//!             self.latency_changed = true
//!         }   
//!     }
//! }
//!
//! struct MyHost;
//! impl HostHandlers for MyHost {
//!     type Shared<'a> = MyHostShared;
//!
//!     type MainThread<'a> = MyHostMainThread<'a>;
//!     type AudioProcessor<'a> = ();
//!
//!     fn declare_extensions(builder: &mut HostExtensions<Self>, shared: &Self::Shared<'_>) {
//!         builder.register::<HostLatency>();
//!     }
//! }
//! ```
//!
//! # Creating custom extensions
//!
//! TODO: document custom extensions.
//!
//! ## Example
//!
//! ```
//! use clack_host::extensions::prelude::*;
//! use clap_sys::ext::latency::{clap_host_latency, clap_plugin_latency, CLAP_EXT_LATENCY};
//! use std::ffi::CStr;
//!
//! /// The type we will receive from a plugin implementing the Latency extension
//! #[derive(Copy, Clone)]
//! pub struct PluginLatency(RawExtension<PluginExtensionSide, clap_plugin_latency>);
//!
//! // Mark this type as being the plugin side of an extension, and tie it to its ID
//! unsafe impl Extension for PluginLatency {
//!     const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_LATENCY];
//!     type ExtensionSide = PluginExtensionSide;
//!
//!     #[inline]
//!     unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
//!         Self(unsafe { raw.cast() })
//!     }
//! }
//!
//! /// The type we will expose to a plugin by implementing the Latency extension
//! #[derive(Copy, Clone)]
//! pub struct HostLatency(RawExtension<HostExtensionSide, clap_host_latency>);
//!
//! // Mark this type as being the host side of an extension, and tie it to its ID
//! unsafe impl Extension for HostLatency {
//!     const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_LATENCY];
//!     type ExtensionSide = HostExtensionSide;
//!
//!     #[inline]
//!     unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
//!         Self(raw.cast())
//!     }
//! }
//!
//! // Implement calling to the plugin-side
//! impl PluginLatency {
//!     // The `clap_plugin_latency.get` function requires to be called on the `[main-thread]`.
//!     // Therefore, we will require the `PluginMainThreadHandle` to be passed.
//!     #[inline]
//!     pub fn get(&self, plugin: &mut PluginMainThreadHandle) -> u32 {
//!         match plugin.use_extension(&self.0).get {
//!             None => 0,
//!             Some(get) => unsafe { get(plugin.as_raw()) },
//!         }
//!     }
//! }
//!
//! /// Provides the implementation of the host-side to be called by the plugin.
//! pub trait HostLatencyImpl {
//!     fn changed(&mut self);
//! }
//!
//! // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
//! unsafe impl<H: for<'a> HostHandlers<MainThread<'a>: HostLatencyImpl>> ExtensionImplementation<H> for HostLatency
//! {
//!     const IMPLEMENTATION: RawExtensionImplementation =
//!         RawExtensionImplementation::new(&clap_host_latency {
//!             changed: Some(changed::<H>),
//!         });
//! }
//!
//! unsafe extern "C" fn changed<H: for<'a> HostHandlers<MainThread<'a>: HostLatencyImpl>>(host: *const clap_host)
//! {
//!     HostWrapper::<H>::handle(host, |host| {
//!         host.main_thread().as_mut().changed();
//!         Ok(())
//!     });
//! }
//! ```

pub use clack_common::extensions::*;
pub mod wrapper;

/// A prelude which re-exports all the types and traits used for custom extension implementation.
///
/// See the [module docs](self) for more information on how to implement custom extensions in a host.
pub mod prelude {
    pub use crate::{
        extensions::wrapper::{HostWrapper, HostWrapperError},
        extensions::{
            Extension, ExtensionImplementation, HostExtensionSide, PluginExtensionSide,
            RawExtension, RawExtensionImplementation,
        },
        host::{HostError, HostHandlers},
        plugin::{
            InactivePluginMainThreadHandle, PluginAudioProcessorHandle, PluginMainThreadHandle,
            PluginSharedHandle,
        },
        utils::ClapId,
    };
    pub use clap_sys::host::clap_host;
}
