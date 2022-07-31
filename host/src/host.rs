//! Core types and traits to implement a Clack host.
//!
//! [`Host`] is the main trait required to be implemented for a
//! Clack host. It can also be associated to two more types,
//! implementing [`HostMainThread`] and [`HostShared`],
//! following [the CLAP thread model, as described
//! here](../../clack_plugin/plugin/index.html#thread-model).
mod error;
mod info;

pub use error::HostError;
pub use info::HostInfo;

use crate::extensions::HostExtensions;
use crate::plugin::{PluginMainThreadHandle, PluginSharedHandle};

/// Representation of the audio context
///
/// Since all the audio processing is done on the plugin side, the host
/// audio processor doesn't need to do anything, unless some extension
/// requires the host to take an action specific to the audio context.
pub trait HostAudioProcessor<'a>: Send + 'a {}

/// Representation of the main context
pub trait HostMainThread<'a>: 'a {
    #[inline]
    #[allow(unused)]
    fn instantiated(&mut self, instance: PluginMainThreadHandle<'a>) {}
}

/// Shared information between the audio and main contexts
pub trait HostShared<'a>: Send + Sync {
    #[inline]
    #[allow(unused)]
    fn instantiated(&mut self, instance: PluginSharedHandle<'a>) {}

    /// Used by the plugin to request that the host deactivate and reactivate
    /// the plugin. The operation may be delayed by the host.
    fn request_restart(&self);

    /// Used by the plugin to request that the host activate and start
    /// processing the plugin. For example, a plugin with external I/O can use
    /// this to wake the plugin up from "sleep".
    fn request_process(&self);

    /// Used by the plugin to request that the host schedule a call to
    /// [`PluginMainThread::on_main_thread()`](../../clack_plugin/plugin/trait.PluginMainThread.html#method.on_main_thread)
    /// on the main thread.
    fn request_callback(&self);
}

/// Container trait that combines [`HostAudioProcessor`],
/// [`HostMainThread`], and [`HostShared`].
///
/// Also used to declare extensions. Implementing this trait is what
/// defines a CLAP host.
///
/// ```rust
/// use clack_host::host::{
///     Host, HostShared, HostAudioProcessor,
/// };
///
/// pub struct Shared;
/// impl HostShared<'_> for Shared {
///     fn request_restart(&self) {
///         // deactivate and reactivate plugin (implementation dependent)
///     }
///     fn request_process(&self) {
///         // process plugin (implementation dependent)
///     }
///     fn request_callback(&self) {
///         // call plugin's `on_main_thread()` (implementation dependent)
///     }
/// }
///
/// impl<'a> Host<'a> for ClapHost {
///     type AudioProcessor = ();
///     type Shared = Shared;
///     type MainThread = ();
///
///     fn declare_extensions(builder: &mut HostExtensions<'_, Self>, _: &Self::Shared);
/// }
/// ```
pub trait Host<'a>: 'static {
    type AudioProcessor: HostAudioProcessor<'a> + 'a;
    type Shared: HostShared<'a> + 'a;
    type MainThread: HostMainThread<'a> + 'a;

    /// To use a host extension, first implement the extension's trait, then
    /// declare the extension using this function.
    ///
    /// ```rust
    /// use clack_host::host::Host;
    /// # use clack_host::host::{
    /// #     HostShared, HostAudioProcessor,
    /// # };
    /// use clack_extensions::log::{
    ///     Log, LogSeverity, implementation::HostLog,
    /// };
    /// # pub struct Shared;
    /// # impl HostShared<'_> for Shared {
    /// #     fn request_restart(&self) { }
    /// #     fn request_process(&self) { }
    /// #     fn request_callback(&self) { }
    /// # }
    ///
    /// impl<'a> Host<'a> for ClapHost {
    ///     // ...
    /// #    type AudioProcessor = ();
    /// #    type Shared = Shared;
    /// #    type MainThread = ();
    ///
    ///     fn declare_extensions(builder: &mut HostExtensions<'_, Self>, _: &Self::Shared) {
    ///         builder.register::<Log>();
    ///     }
    /// }
    ///
    /// impl HostLog for SH {
    ///     fn log(&self, severity: clack_extensions::log::LogSeverity, message: &str) {
    ///         match severity {
    ///             LogSeverity::Debug   => println!("debug: {message}"),
    ///             LogSeverity::Info    => println!("info: {message}"),
    ///             LogSeverity::Warning => println!("warn: {message}"),
    ///             LogSeverity::Error   => println!("error: {message}"),
    ///             LogSeverity::Fatal   => println!("fatal: {message}"),
    ///             LogSeverity::HostMisbehaving   => println!("host misbehaving: {message}"),
    ///             LogSeverity::PluginMisbehaving => println!("plugin misbehaving: {message}"),
    ///         }
    ///     }
    /// }
    /// ```
    #[inline]
    #[allow(unused)]
    fn declare_extensions(builder: &mut HostExtensions<Self>, shared: &Self::Shared) {}
}

// QoL implementations

impl<'a> HostAudioProcessor<'a> for () {}
impl<'a> HostMainThread<'a> for () {}
