//! Core types and traits to implement a Clack plugin.
//!
//! The [`PluginAudioProcessor`] trait is the main one required to be implemented for a Clack plugin. It
//! can also be associated to two more types, implementing [`PluginMainThread`] and [`PluginShared`],
//! following the CLAP thread model, as described below.
//!
//! # Thread model
//!
//! CLAP's thread model for plugins is split into three classes of operations: those happening in an
//! audio processing thread, those happening in the main thread, and thread-safe operations:
//!
//! * The *audio thread* (`[audio-thread]` in the CLAP specification): this is represented by a type
//! implementing the main [`PluginAudioProcessor`] trait (also named the audio processor), which is [`Send`] but
//! [`!Sync`](Sync), and is the only one required to implement a Clack plugin.
//!
//!   This type handles all DSP in one of the host's audio threads, of which there may be
//!   multiple, if the host uses a thread pool for example.
//!
//!   The host is free to [`Send`] the [`PluginAudioProcessor`] type between any of its audio threads, but any
//!   operation of this class is guaranteed to be exclusive (`&mut`) to a single audio thread.
//!
//!   One exception is for CLAP plugins' activation and deactivation (represented in Clack by the
//!   plugin type's construction and destruction), which is guaranteed to happen in the Main Thread
//!   instead. This allows the plugin's [`activate`](PluginAudioProcessor::activate) and
//!   [`deactivate`](PluginAudioProcessor::deactivate) methods to receive temporary exclusive references to the
//!   main thread type during its construction and destruction.
//!
//! * The *main thread* (`[main-thread]` in the CLAP specification): this is represented by a type
//!   implementing the [`PluginMainThread`] trait, which is neither [`Send`] nor [`Sync`]. If
//!   main-thread operations are not needed by a plugin implementation, `()` can be used instead.
//!
//!   This type can handle all non-thread-safe operations, such as those related to GUI handling,
//!   however extensions can extend its use to more kinds of operations.
//!
//!   The host cannot [`Send`] this type to any other threads, and this type has to be constructed
//!   before any other operation is done on the plugin.
//!
//! * *Thread-safe operations* (`[thread-safe]` in the CLAP specification) are represented by a type
//!   implementing the [`PluginShared`] trait, which is both [`Send`] and [`Sync`], and will be
//!   shared between the main thread and the audio thread. If it isn't needed , `()` can be used
//!   instead.
//!
//!   It can be used to hold read-only data (such as all of the detected host extensions), or to
//!   hold any other kind of synchronized state.
//!
//!   However, it should be noted that this type *can* be used by the host simultaneously from
//!   threads that are neither the main thread nor the audio thread.

use crate::extensions::PluginExtensions;
use crate::host::{HostAudioThreadHandle, HostHandle, HostMainThreadHandle};
use crate::process::Audio;
use crate::process::Events;
use crate::process::Process;
use clack_common::process::ProcessStatus;

pub mod descriptor;
mod error;
mod instance;
pub(crate) mod logging;

use crate::plugin::descriptor::PluginDescriptor;
pub use error::PluginError;
pub use instance::*;

/// The part of the data and operations of a plugin that are thread-safe.
///
/// The associated lifetime `'a` represents the lifetime of the plugin itself, as well as the
/// lifetime of the data exposed by the host.
///
/// This type requires to be both [`Send`] and [`Sync`]: it can be used simultaneously by multiple
/// threads, including (but not limited to) the main thread and the audio thread.
///
/// See the [module documentation](crate::plugin) for more information on the thread model.
pub trait PluginShared<'a>: Sized + Send + Sync + 'a {
    /// Creates a new instance of this shared data.
    ///
    /// This struct receives a thread-safe host handle that can be stored for the lifetime of the plugin.
    ///
    /// # Errors
    /// This operation may fail for any reason, in which case `Err` is returned and the plugin is
    /// not instantiated.
    fn new(host: HostHandle<'a>) -> Result<Self, PluginError>;
}

impl<'a> PluginShared<'a> for () {
    #[inline]
    fn new(_host: HostHandle<'a>) -> Result<Self, PluginError> {
        Ok(())
    }
}

/// The part of the data and operation of a plugin that must be on the main thread.
///
/// The associated lifetime `'a` represents the lifetime of the plugin itself, as well as the
/// lifetime of the data exposed by the host.
///
/// This type requires neither [`Send`] nor [`Sync`]: it is guaranteed to stay on the main thread
/// at all times.
///
/// See the [module documentation](crate::plugin) for more information on the thread model.
pub trait PluginMainThread<'a, S: PluginShared<'a>>: Sized + 'a {
    /// Creates a new instance of the plugin's main thread.
    ///
    /// This struct receives an exclusive host handle that can be stored for the lifetime of the plugin.
    ///
    /// # Errors
    /// This operation may fail for any reason, in which case `Err` is returned and the plugin is
    /// not instantiated.
    fn new(host: HostMainThreadHandle<'a>, shared: &'a S) -> Result<Self, PluginError>;

    /// This is called by the host on the main thread, in response to a previous call to
    /// [`HostHandle::request_callback`](HostHandle::request_callback).
    ///
    /// The default implementation of this method does nothing.
    #[inline]
    fn on_main_thread(&mut self) {}
}

impl<'a, S: PluginShared<'a>> PluginMainThread<'a, S> for () {
    #[inline]
    fn new(_host: HostMainThreadHandle<'a>, _shared: &'a S) -> Result<Self, PluginError> {
        Ok(())
    }
}

/// The audio configuration passed to a plugin's audio processor upon activation.
///
/// This is guaranteed to remain constant and valid throughout the audio processor's lifetime,
/// until deactivation.
#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub struct AudioConfiguration {
    /// The audio's sample rate.
    pub sample_rate: f64,
    /// The minimum amount of samples that will be [processed](PluginAudioProcessor::process) at once.
    pub min_sample_count: u32,
    /// The maximum amount of samples that will be [processed](PluginAudioProcessor::process) at once.
    pub max_sample_count: u32,
}

pub trait Plugin: 'static {
    /// The type holding the plugin's data and operations that belong to the audio thread.
    ///
    /// See the [module documentation](crate::plugin) for more information on the thread model.
    type AudioProcessor<'a>: PluginAudioProcessor<'a, Self::Shared<'a>, Self::MainThread<'a>>;

    /// The type holding the plugin's thread-safe data and operations.
    ///
    /// If not needed, the empty `()` type can be used instead.
    ///
    /// See the [module documentation](crate::plugin) for more information on the thread model.
    type Shared<'a>: PluginShared<'a>;

    /// The type holding the plugin's data and operations that belong to the main thread.
    ///
    /// If not needed, the empty `()` type can be used instead.
    ///
    /// See the [module documentation](crate::plugin) for more information on the thread model.
    type MainThread<'a>: PluginMainThread<'a, Self::Shared<'a>>;

    /// Creates a new Plugin Descriptor.
    ///
    /// This contains read-only data about the plugin, such as it's name, stable identifier, and more.
    ///
    /// See the [`PluginDescriptor`] trait's documentation for more information.
    fn get_descriptor() -> Box<dyn PluginDescriptor>;

    #[inline]
    #[allow(unused_variables)]
    fn declare_extensions(builder: &mut PluginExtensions<Self>, shared: &Self::Shared<'_>) {}
}

/// The audio processor and main part of a plugin.
///
/// This type implements all DSP-related operations, most notably [`process`](PluginAudioProcessor::process),
/// which processes all input and output audio and events.
///
/// The associated lifetime `'a` represents the lifetime of the plugin itself, as well as the
/// lifetime of the data exposed by the host.
///
/// This type requires to be [`Send`] but not [`Sync`]: it can be sent between any of the host's
/// threads, but none of its operations will be performed on multiple threads simultaneously.
///
/// The audio processor can also define two associated types, [`Shared`](Plugin::Shared) and
/// [`MainThread`](Plugin::MainThread), allowing to execute operations in and hold data belonging
/// to other threads. If they are not needed, the empty `()` type can be used instead, for convenience.
///
/// See the [module documentation](crate::plugin) for more information on the thread model.
pub trait PluginAudioProcessor<'a, S: PluginShared<'a>, M: PluginMainThread<'a, S>>:
    Sized + Send + 'a
{
    /// Creates and activates the audio processor.
    ///
    /// This method serves as a constructor for the audio processor, in which it can perform
    /// non-realtime-safe initialization operations, such as allocating audio buffers using the
    /// provided [`AudioConfiguration`].
    ///
    /// This method is always executed on the main thread, allowing it to temporarily access main
    /// thread data.
    ///
    /// # Arguments
    ///
    /// * `host`: an exclusive host handle that can be stored for the lifetime of the plugin.
    /// * `main_thread`: a temporary exclusive reference to the plugin's main thread data.
    /// * `shared`: a reference to the plugin's shared data, that can be stored for the lifetime of the plugin.
    /// * `audio_config`: the [`AudioConfiguration`], valid throughout the audio processor's lifetime.
    ///
    /// # Errors
    ///
    /// If the plugin's audio processor was already activated, this method should return a
    /// [`PluginError::AlreadyActivated`] error. This is a fatal error which only possible due to a
    /// faulty host, and should be considered to be a bug. In this case, the plugin's activation is
    /// aborted.
    ///
    /// In addition, this operation may fail for any other reason, in which case `Err` is returned
    /// and the plugin is not instantiated.
    ///
    /// # Realtime Safety
    ///
    /// This method is not realtime-safe: it may perform memory allocations of audio buffers, or any
    /// other initialization the plugin may deem necessary.
    fn activate(
        host: HostAudioThreadHandle<'a>,
        main_thread: &mut M,
        shared: &'a S,
        audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError>;

    fn process(
        &mut self,
        process: Process,
        audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError>;

    /// Deactivates the audio processor.
    ///
    /// This method can serve as a destructor for the audio processor.
    ///
    /// This method is always executed on the main thread, allowing it to temporarily access main
    /// thread data.
    ///
    /// # Arguments
    ///
    /// * `main_thread`: a temporary exclusive reference to the plugin's main thread data.
    ///
    /// # Realtime Safety
    ///
    /// This method is not realtime-safe: it may perform memory de-allocations of audio buffers, or
    /// any other un-initialization the plugin may deem necessary.
    #[allow(unused)]
    #[inline]
    fn deactivate(self, main_thread: &mut M) {}

    #[allow(unused)]
    #[inline]
    fn reset(&mut self) {}

    #[inline]
    fn start_processing(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    #[inline]
    fn stop_processing(&mut self) {}
}

impl<'a, M: PluginMainThread<'a, S>, S: PluginShared<'a>> PluginAudioProcessor<'a, S, M> for () {
    #[inline]
    fn activate(
        _host: HostAudioThreadHandle<'a>,
        _main_thread: &mut M,
        _shared: &'a S,
        _audio_config: AudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(())
    }

    #[inline]
    fn process(
        &mut self,
        _process: Process,
        _audio: Audio,
        _events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        Ok(ProcessStatus::Sleep)
    }
}
