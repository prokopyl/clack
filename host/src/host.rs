#![deny(missing_docs)]

//! Core types and traits to implement a Clack host.
//!
//! The [`Host`] trait is the main one required to be implemented for a Clack plugin. It provides
//! the host's supported extensions and is associated to a main type implementing [`HostShared`] ,
//! as well as two more optional types implementing [`HostMainThread`] and [`HostAudioProcessor`] (the
//! unit type `()` can be used as default implementations that do nothing).
//!
//! These three types must implement host interfaces to allow them to respond to various requests
//! from the plugin, following the CLAP thread model described below.
//!
//! # Thread specifications
//!
//! CLAP's thread model for plugins is split into three specifications of operations: those
//! happening in the main thread, those happening in an audio processing thread,
//! and thread-safe operations:
//!
//! * The *main thread* (`[main-thread]` in the CLAP specification): this is represented by a type
//!   implementing the [`HostMainThread`] trait, which is neither [`Send`] nor [`Sync`], and lives
//!   encapsulated in the [`PluginInstance`](crate::plugin::PluginInstance) struct.
//!
//!   This type can handle all non-realtime-safe operations, such as those related to buffer
//!   allocations or GUI handling, and extensions can extend its use to more kinds of operations.
//!
//!   This type is intended to stay on the thread it was created in (i.e. the "main" thread):
//!   implementations cannot [`Send`] this type to any other threads.
//!
//! * The *audio thread* (`[audio-thread]` in the CLAP specification): this is represented by a type
//!   implementing the [`HostAudioProcessor`] trait, which is [`Send`] but
//!   [`!Sync`](Sync), and lives encapsulated in the
//!   [`PluginAudioProcessor`](crate::process::PluginAudioProcessor) struct. It is only
//!   instantiated when the [`activate`](crate::plugin::PluginInstance::activate) method is called,
//!   and is dropped on [`deactivate`](crate::plugin::PluginInstance::deactivate). If it isn't
//!   needed , `()` can be used instead.
//!
//!   This type is designed to handle all DSP-related requests from the plugin and lives in one of
//!   the host's audio threads, of which there may be multiple, if the host uses a thread pool for
//!   example.
//!
//!   The host is free to [`Send`] the [`HostAudioProcessor`] type between any of its audio threads,
//!   but any operation of this class is guaranteed to be called by only a single thread
//!   simultaneously ([`!Sync`](Sync)).
//!
//!   One exception is for CLAP plugins' activation and deactivation (represented in Clack by the
//!   plugin type's construction and destruction), which is guaranteed to happen in the Main Thread
//!   instead. This allows an [`HostAudioProcessor`] implementation to receive temporary exclusive
//!   references to the [`HostMainThread`] type during its construction and destruction, to take
//!   and release ownership of extra buffers for instance.
//!
//! * *Thread-safe operations* (`[thread-safe]` in the CLAP specification) are represented by a type
//!   implementing the [`HostShared`] trait, which is both [`Send`] and [`Sync`], and will be
//!   shared between the main thread and the audio thread.
//!
//!   It can be used to hold read-only data (such as all of the detected plugin extensions), or to
//!   hold any other kind of state that is to be synchronized between multiple threads.
//!
//!   However, it should be noted that this type *can* be used by the plugin simultaneously from
//!   threads that are neither the main thread nor the audio thread.
//!
//! # Example
//!
//! This example implements a basic host which is able to process callback requests from the plugin,
//! along with two extensions: `latency` and `log`.
//!
//! This is done by implementing the [`Host`] trait and specifying its associated traits:
//! [`HostShared`], [`HostMainThread`], and [`HostAudioProcessor`].
//!
//! Because our host supports some extensions, we also implement the [`Host::declare_extensions`]
//! method to declare them to the plugin, which requires us to implement the associated traits on
//! the appropriate [`Host`] associated types to handle the host-side.
//!
//! For more information about which extension traits needs to be implemented on which time, refer
//! to that extension's documentation.
//!
//! For more information about how to work with extensions, see the
//! [`extensions`](crate::extensions) module documentation.
//!
//! ```
//! use clack_host::events::event_types::*;
//! use clack_host::prelude::*;
//!
//! use clack_extensions::latency::*;
//! use clack_extensions::log::*;
//!
//! use std::sync::atomic::{AtomicBool, Ordering};
//! use std::sync::OnceLock;
//! use std::ffi::CStr;
//!
//! #[derive(Default)]
//! struct MyHostShared<'a> {
//!     // A real-world implementation may use a fancier notification system.
//!     // For this example, we are simply checking a handful of atomics from time to time.
//!     restart_requested: AtomicBool,
//!     process_requested: AtomicBool,
//!     callback_requested: AtomicBool,
//!
//!     // Queried extensions
//!     // Note this may be None even after instantiation,
//!     // in case the extension isn't supported by the plugin.
//!     latency_extension: OnceLock<Option<&'a PluginLatency>>
//! }
//!
//! impl<'a> HostShared<'a> for MyHostShared<'a> {
//!     // Once the plugin is fully instantiated, we can query its extensions
//!     fn instantiated(&self, instance: PluginSharedHandle<'a>) {
//!         let _ = self.latency_extension.set(instance.get_extension());
//!     }
//!     
//!     fn request_restart(&self) { self.restart_requested.store(true, Ordering::SeqCst) }
//!     fn request_process(&self) { self.process_requested.store(true, Ordering::SeqCst) }
//!     fn request_callback(&self) { self.callback_requested.store(true, Ordering::SeqCst) }
//! }
//!
//! impl<'a> HostLogImpl for MyHostShared<'a> {
//!     fn log(&self, severity: LogSeverity, message: &str) {
//!         // A real-world implementation would make sure this is wait-free.
//!         // But for this example, println! is good enough.
//!         println!("[{severity}] [Plugin] {message}")
//!     }
//! }
//!
//! struct MyHostMainThread<'a> {
//!     shared: &'a MyHostShared<'a>,
//!     instance: Option<PluginMainThreadHandle<'a>>,
//!
//!     reported_latency: Option<u32>
//! }
//!
//! impl<'a> HostMainThread<'a> for MyHostMainThread<'a> {
//!     fn instantiated(&mut self, instance: PluginMainThreadHandle<'a>) {
//!         self.instance = Some(instance);
//!     }
//! }
//!
//! impl<'a> HostLatencyImpl for MyHostMainThread<'a> {
//!     fn changed(&mut self) {
//!         if let (Some(Some(latency)), Some(instance)) = (self.shared.latency_extension.get(), &mut self.instance) {
//!             self.reported_latency = Some(latency.get(instance));
//!         }   
//!     }
//! }
//!
//! struct MyHost;
//! impl Host for MyHost {
//!     type Shared<'a> = MyHostShared<'a>;
//!
//!     type MainThread<'a> = MyHostMainThread<'a>;
//!     type AudioProcessor<'a> = ();
//!
//!     fn declare_extensions(builder: &mut HostExtensions<Self>, shared: &Self::Shared<'_>) {
//!         builder
//!             .register::<HostLog>()
//!             .register::<HostLatency>();
//!     }
//! }
//!
//! # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Information about our totally legit host.
//! let host_info = HostInfo::new("Legit Studio", "Legit Ltd.", "https://example.com", "4.3.2")?;
//!
//! # mod diva { include!("./bundle/diva_stub.rs"); }
//! # let bundle = unsafe { PluginBundle::load_from_raw(&diva::DIVA_STUB_ENTRY, "/home/user/.clap/u-he/libdiva.so")? };
//! # #[cfg(never)]
//! let bundle = PluginBundle::load("/home/user/.clap/u-he/libdiva.so")?;
//!
//! let mut plugin_instance = PluginInstance::<MyHost>::new(
//!     |_| MyHostShared::default(),
//!     |shared| MyHostMainThread { shared, instance: None, reported_latency: None },
//!     &bundle,
//!     // We're hard-coding a specific plugin to load for this example
//!     CStr::from_bytes_with_nul(b"com.u-he.diva\0")?,
//!     &host_info
//! )?;
//!
//! // Assume we've activated the plugin and just done some processing.
//! /* ... */
//!
//! // Let's check if the plugin requested a callback, by accessing our shared host data.
//! let shared: &MyHostShared = plugin_instance.shared_host_data();
//!
//! // This fetches the previous value and sets it to false in a single atomic operation.
//! if shared.callback_requested.fetch_and(false, Ordering::SeqCst) {
//!     plugin_instance.call_on_main_thread_callback();
//! }
//!
//! // Do the same for the restart and process requests
//! /* ... */
//!
//! # Ok(()) }
//! ```

mod error;
mod extensions;
mod info;

pub use error::HostError;
pub use extensions::HostExtensions;
pub use info::HostInfo;

use crate::plugin::{PluginMainThreadHandle, PluginSharedHandle};

/// Host data and callbacks that are tied to `[main-thread]` operations.
///
/// This trait requires neither [`Send`] nor [`Sync`], as types implementing it are intended to
/// contain host data staying on the main thread.
///
/// This trait is bound to the plugin instance's lifetime (`'a`).
///
/// See the [module docs](self) for more information, and for an example implementation.
pub trait HostMainThread<'a>: 'a {
    /// Called when the plugin has been successfully instantiated.
    ///
    /// This is given a handle to the plugin's own main thread data ([`PluginMainThreadHandle`]),
    /// which can be used to call plugin callbacks, and can be kept for the remainder of the
    /// plugin instance's lifetime.
    #[inline]
    #[allow(unused)]
    fn instantiated(&mut self, instance: PluginMainThreadHandle<'a>) {}
}

/// Host data and callbacks that are tied to `[audio-thread]` operations.
///
/// This trait requires [`Send`], as a plugin's audio processor can be sent between multiple
/// threads, but can only be accessed by one thread at a time.
///
/// This trait is bound to the plugin audio processor's lifetime (`'a`).
///
/// This trait doesn't have any inherent methods, and serves only to restrict the lifetime and
/// [`Send`] bounds. However, extensions can add more callbacks to the [`Host::AudioProcessor`]
/// type.
///
/// See the [module docs](self) for more information, and for an example implementation.
pub trait HostAudioProcessor<'a>: Send + 'a {}

/// Host data and callbacks that are tied to `[thread-safe]` operations.
///
/// This trait requires both [`Send`] *and* [`Sync`], as those callbacks and data can be used from
/// multiple different threads at the same time.
///
/// This trait is bound to the plugin instance's lifetime (`'a`).
///
/// See the [module docs](self) for more information, and for an example implementation.
pub trait HostShared<'a>: Send + Sync {
    /// Called when the plugin has been successfully instantiated.
    ///
    /// This is given a handle to the plugin's own shared data ([`PluginSharedHandle`]),
    /// which can be used to call plugin callbacks, and can be kept for the remainder of the
    /// plugin instance's lifetime.
    #[inline]
    #[allow(unused)]
    fn instantiated(&self, instance: PluginSharedHandle<'a>) {}

    /// Called by the plugin when it requests to be deactivated and then restarted by the host.
    ///
    /// This operation may be delayed by the host.
    fn request_restart(&self);

    /// Called by the plugin when it requests to be activated and/or to start processing.
    fn request_process(&self);

    /// Called by the plugin when it requests a call to the
    /// [`on_main_thread` callback](crate::plugin::PluginInstance::call_on_main_thread_callback)
    /// to be scheduled on the main thread.
    fn request_callback(&self);
}

/// A Clack Host implementation.
///
/// This type does not hold any data nor is used at runtime. It exists only to tie
/// the [`Shared`](Host::Shared), [`Shared`](Host::Shared), and [`Shared`](Host::Shared)
/// associated types together with the declared extensions.
///
/// See the [module docs](self) for more information, and for an example implementation.
pub trait Host: 'static {
    /// The type that handles host data and callbacks tied to `[thread-safe]` operations.
    /// See the [`HostShared`] docs and the [module docs](self) for more information.
    type Shared<'a>: HostShared<'a> + 'a;

    /// The type that handles host data and callbacks tied to `[main-thread]` operations.
    /// See the [`HostMainThread`] docs and the [module docs](self) for more information.
    type MainThread<'a>: HostMainThread<'a> + 'a;

    /// The type that handles host data and callbacks tied to `[audio-thread]` operations.
    /// See the [`HostAudioProcessor`] docs and the [module docs](self) for more information.
    type AudioProcessor<'a>: HostAudioProcessor<'a> + 'a;

    /// Declares all of the extensions supported by this host.
    ///
    /// Extension declaration is done using the [`HostExtensions::register`] method.
    ///
    /// A temporary reference to this host's [`Shared`](Host::Shared) type is also given, in case
    /// extensions need to be dynamically declared.
    #[inline]
    #[allow(unused)]
    fn declare_extensions(builder: &mut HostExtensions<Self>, shared: &Self::Shared<'_>) {}
}

// QoL implementations

impl<'a> HostAudioProcessor<'a> for () {}
impl<'a> HostMainThread<'a> for () {}
impl<'a> HostShared<'a> for () {
    fn request_restart(&self) {}
    fn request_process(&self) {}
    fn request_callback(&self) {}
}
impl Host for () {
    type Shared<'a> = ();
    type MainThread<'a> = ();
    type AudioProcessor<'a> = ();
}
