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
//!   implementing the main [`PluginAudioProcessor`] trait (also named the audio processor), which is [`Send`] but
//!   [`!Sync`](Sync), and is the only one required to implement a Clack plugin.
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
use crate::host::HostAudioProcessorHandle;
use crate::process::{Audio, Events, PluginAudioConfiguration, Process, ProcessStatus};

mod error;
mod instance;
pub(crate) mod logging;

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

impl PluginShared<'_> for () {}

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

/// The main trait required to implement a CLAP plugin.
///
/// Types implementing this trait are never instantiated and are not meant to contain any data.
///
/// Their purpose is to tie together the various subtypes that will be instantiated and live across
/// different threads.
///
/// # Plugin instantiation
///
/// Plugin instantiation and initialization is not covered by this trait. The [`Shared`] and
/// [`MainThread`] components are created from a [`PluginFactory`] implementation
/// (in [`PluginFactory::create_plugin`]) using the [`PluginInstance::new`] method.
///
/// Alternatively, if custom [`PluginFactory`] and [`Entry`] implementations are not needed, one
/// can use a [`SinglePluginEntry`] and implement its companion trait [`DefaultPluginFactory`]
/// to implement the instantiation instead.
///
/// [`Shared`]: Self::Shared
/// [`MainThread`]: Self::MainThread
/// [`PluginFactory`]: crate::factory::plugin::PluginFactoryImpl
/// [`PluginFactory::create_plugin`]: crate::factory::plugin::PluginFactoryImpl::create_plugin
/// [`Entry`]: crate::entry::Entry
/// [`SinglePluginEntry`]: crate::entry::SinglePluginEntry
/// [`DefaultPluginFactory`]: crate::entry::DefaultPluginFactory
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

    /// Declares the extensions this plugin supports.
    ///
    /// This is implemented by calling [`register`] on the given [`PluginExtensions`]
    /// builder for every extension type that is supported.
    ///
    /// A reference to the [`Shared`](Self::Shared) type is also given. However, it can be `None`,
    /// as the host is allowed to query extensions before the plugin has finished initializing.
    ///
    /// [`register`]: PluginExtensions::register
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
        host: HostAudioProcessorHandle<'a>,
        main_thread: &mut M,
        shared: &'a S,
        audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError>;

    /// Processes a chunk of audio samples and events.
    ///
    /// This method returns a [`ProcessStatus`] as a hint towards whether the host can set this
    /// plugin to sleep, or if the plugin wants to process more audio or events.
    /// This is only a hint however, and the host is free to ignore it.
    ///
    /// # Arguments
    ///
    /// * `process` contains metadata about the current process call, including transport information and a steady sample counter.
    /// * `audio` contains references to all the audio buffers for this block.
    /// * `events` contains both the [`InputEvents`](crate::prelude::InputEvents) list and the [`OutputEvents`](crate::prelude::OutputEvents) queue.
    ///
    /// # Realtime Safety
    ///
    /// This method *MUST* be realtime-safe, and is in fact the most performance-sensitive part
    /// of the whole plugin implementation.
    ///
    /// # Errors
    ///
    /// This method may fail for any reason, depending on the plugin's implementation.
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

    /// Resets the plugin's audio processing state.
    ///
    /// This clears all the plugin's internal buffers, kills all voices, and resets all processing
    /// state such as envelopes, LFOs, oscillators, filters, etc.
    ///
    /// Calling this method allows the `steady_time` parameter passed to [`process`](Self::process)
    /// to jump backwards.
    #[allow(unused)]
    #[inline]
    fn reset(&mut self) {}

    /// Starts the plugin's continuous audio processing.
    ///
    /// This is called when the plugin needs to wake up from sleep, or after it was [activated](Self::activate).
    ///
    /// # Errors
    ///
    /// This method may fail for any reason, depending on the plugin's implementation.
    #[inline]
    fn start_processing(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    /// Stops the plugin's continuous audio processing, indicating it is being sent to sleep.
    ///
    /// When this is called, the plugin's audio processor cannot assume that the next block of audio
    /// and events it receives (if any) is contiguous to the last one it [processed](Self::process).
    #[inline]
    fn stop_processing(&mut self) {}
}

impl<'a, M: PluginMainThread<'a, S>, S: PluginShared<'a>> PluginAudioProcessor<'a, S, M> for () {
    #[inline]
    fn activate(
        _host: HostAudioProcessorHandle<'a>,
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
