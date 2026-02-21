use crate::host::HostHandlers;
use crate::process::ProcessingStartError;
use core::fmt;
use core::fmt::{Debug, Display, Formatter};
use std::error::Error;

/// All errors that can arise using plugin instances.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PluginInstanceError {
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
    /// Tried to instantiate a plugin from an entry which lacks a [`PluginFactory`](crate::factory::plugin::PluginFactory).
    ///
    /// This is a sign of a misbehaving plugin implementation.
    MissingPluginFactory,
    /// The plugin's instantiation failed.
    InstantiationFailed,
    /// The plugin has already been destroyed.
    PluginDestroyed,
    /// The plugin's audio processing failed.
    ProcessingFailed,
    /// Tried to perform or stop processing when the audio processor was not started yet.
    ProcessingStopped,
    /// Tried to start processing when the processing was already started.
    ProcessingStarted,
    /// The underlying plugin's `create_plugin` C function was a null pointer.
    ///
    /// This is a sign of a misbehaving plugin implementation.
    NullFactoryCreatePluginFunction,
    /// The underlying plugin's `process` C function was a null pointer.
    ///
    /// This is a sign of a misbehaving plugin implementation.
    NullProcessFunction,
    /// The underlying plugin's `activate` C function was a null pointer.
    ///
    /// This is a sign of a misbehaving plugin implementation.
    NullActivateFunction,
}

impl PluginInstanceError {
    pub(crate) fn msg(&self) -> &'static str {
        match self {
            Self::StartProcessingFailed => "Could not start processing",
            Self::AlreadyActivatedPlugin => "Plugin was already activated",
            Self::StillActivatedPlugin => {
                "Attempted to deactivate Plugin which still has an active AudioProcessor"
            }
            Self::DeactivatedPlugin => "Plugin is currently deactivated",
            Self::ActivationFailed => "Unable to activate",
            Self::PluginNotFound => "Specified plugin was not found",
            Self::MissingPluginFactory => "No plugin factory was provided",
            Self::InstantiationFailed => "Could not instantiate",
            Self::PluginDestroyed => "Plugin was destroyed",
            Self::ProcessingFailed => "Could not process",
            Self::ProcessingStopped => "Audio Processor is currently stopped",
            Self::ProcessingStarted => "Audio Processor is currently started",
            Self::NullProcessFunction => "Plugin's process function is null",
            Self::NullActivateFunction => "Plugin's activate function is null",
            Self::NullFactoryCreatePluginFunction => {
                "Plugin Factory's create_plugin function is null"
            }
        }
    }
}

impl Display for PluginInstanceError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.msg())
    }
}

impl Error for PluginInstanceError {}

impl<H: HostHandlers> From<ProcessingStartError<H>> for PluginInstanceError {
    #[inline]
    fn from(_: ProcessingStartError<H>) -> Self {
        Self::StartProcessingFailed
    }
}
