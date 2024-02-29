use crate::host::Host;
use crate::process::ProcessingStartError;
use core::fmt;

/// All errors that can arise using plugin instances.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    /// The plugin's audio processing could not be started.
    StartProcessingFailed,
    /// Tried to activate a plugin that was already activated.
    AlreadyActivatedPlugin,
    /// Tried to deactivate a plugin instance while its audio processor was still alive.
    StillActivatedPlugin,
    /// Attempted to perform an operation on the plugin instance's audio processor, but it was
    /// not activated yet.
    DeactivatedPlugin,
    /// The plugin instance's audio processor's activation failed.
    ActivationFailed,
    /// No plugin with a matching ID was found during instantiation.
    PluginNotFound,
    /// Tried to instantiate a plugin from a bundle which lacks a [`PluginFactory`](crate::factory::PluginFactory).
    ///
    /// This is a sign of a misbehaving plugin implementation.
    MissingPluginFactory,
    /// The plugin's instantiation failed.
    InstantiationFailed,
    /// The plugin has already been destroyed.
    PluginDestroyed,
    /// The plugin's audio processing failed.
    ProcessingFailed,
    /// The plugin's audio processor handle was poisoned.
    ///
    /// This is only possible if starting or stopping the audio processor panicked or crashed, but
    /// the handle was kept alive.
    ProcessorHandlePoisoned,
    /// Tried to perform or stop processing when the audio processor was not started yet.
    ProcessingStopped,
    /// Tried to start processing when the processing was already started.
    ProcessingStarted,
    /// The underlying plugin's C `create_plugin` C function was a null pointer.
    ///
    /// This is a sign of a misbehaving plugin implementation.
    NullFactoryCreatePluginFunction,
    /// The underlying plugin's C `process` C function was a null pointer.
    ///
    /// This is a sign of a misbehaving plugin implementation.
    NullProcessFunction,
    /// The underlying plugin's C `activate` C function was a null pointer.
    ///
    /// This is a sign of a misbehaving plugin implementation.
    NullActivateFunction,
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::StartProcessingFailed => write!(f, "Could not start processing"),
            Self::AlreadyActivatedPlugin => write!(f, "Plugin was already activated"),
            Self::StillActivatedPlugin => write!(
                f,
                "Attempted to deactivate Plugin which still has an active AudioProcessor"
            ),
            Self::DeactivatedPlugin => write!(f, "Plugin is currently deactivated"),
            Self::ActivationFailed => write!(f, "Unable to activate"),
            Self::PluginNotFound => write!(f, "Specified plugin was not found"),
            Self::MissingPluginFactory => write!(f, "No plugin factory was provided"),
            Self::InstantiationFailed => write!(f, "Could not instantiate"),
            Self::PluginDestroyed => write!(f, "Plugin was destroyed"),
            Self::ProcessingFailed => write!(f, "Could not process"),
            Self::ProcessorHandlePoisoned => write!(f, "Audio Processor handle was poisoned"),
            Self::ProcessingStopped => write!(f, "Audio Processor is currently stopped"),
            Self::ProcessingStarted => write!(f, "Audio Processor is currently started"),
            Self::NullProcessFunction => write!(f, "Plugin's process function is null"),
            Self::NullActivateFunction => write!(f, "Plugin's activate function is null"),
            Self::NullFactoryCreatePluginFunction => {
                write!(f, "Plugin Factory's create_plugin function is null")
            }
        }
    }
}

impl std::error::Error for HostError {}

impl<H: Host> From<ProcessingStartError<H>> for HostError {
    #[inline]
    fn from(_: ProcessingStartError<H>) -> Self {
        Self::StartProcessingFailed
    }
}
