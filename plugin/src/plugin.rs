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
//!   It can be used to hold read-only data (such as all the detected host extensions), or to
//!   hold any other kind of synchronized state.
//!
//!   However, it should be noted that this type *can* be used by the host simultaneously from
//!   threads that are neither the main thread nor the audio thread.

use crate::extensions::PluginExtensions;
use crate::host::HostAudioThreadHandle;
use crate::process::{Audio, Events, PluginAudioConfiguration, Process, ProcessStatus};

mod descriptor;
mod error;
mod instance;
pub(crate) mod logging;

pub use descriptor::*;
pub use error::PluginError;
pub use instance::*;

pub use clack_common::plugin::*;

/// The part of the data and operations of a plugin that are thread-safe.
///
/// The associated lifetime `'a` represents the lifetime of the plugin itself, as well as the
/// lifetime of the data exposed by the host.
///
/// This type requires to be both [`Send`] and [`Sync`]: it can be used simultaneously by multiple
/// threads, including (but not limited to) the main thread and the audio thread.
///
/// See the [module documentation](crate::plugin) for more information on the thread model.
pub trait PluginShared<'a>: Sized + Send + Sync + 'a {}

impl<'a> PluginShared<'a> for () {}

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
    /// This is called by the host on the main thread, in response to a previous call to
    /// [`HostSharedHandle::request_callback`](crate::host::HostSharedHandle::request_callback).
    ///
    /// The default implementation of this method does nothing.
    #[inline]
    fn on_main_thread(&mut self) {}
}

impl<'a, S: PluginShared<'a>> PluginMainThread<'a, S> for () {}

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

    #[inline]
    #[allow(unused_variables)]
    fn declare_extensions(builder: &mut PluginExtensions<Self>, shared: Option<&Self::Shared<'_>>) {
    }
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
    /// provided [`PluginAudioConfiguration`].
    ///
    /// This method is always executed on the main thread, allowing it to temporarily access main
    /// thread data.
    ///
    /// # Arguments
    ///
    /// * `host`: an exclusive host handle that can be stored for the lifetime of the plugin.
    /// * `main_thread`: a temporary exclusive reference to the plugin's main thread data.
    /// * `shared`: a reference to the plugin's shared data, that can be stored for the lifetime of the plugin.
    /// * `audio_config`: the [`PluginAudioConfiguration`], valid throughout the audio processor's lifetime.
    ///
    /// # Errors
    ///
    /// This operation may fail for any reason, in which case `Err` is returned
    /// and the plugin is not activated.
    ///
    /// # Realtime Safety
    ///
    /// This method is not realtime-safe: it may perform memory allocations of audio buffers, or any
    /// other initialization the plugin may deem necessary.
    fn activate(
        host: HostAudioThreadHandle<'a>,
        main_thread: &mut M,
        shared: &'a S,
        audio_config: PluginAudioConfiguration,
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
        _audio_config: PluginAudioConfiguration,
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
