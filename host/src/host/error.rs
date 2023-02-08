use crate::host::Host;
use crate::instance::processor::ProcessingStartError;
use core::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    StartProcessingFailed,
    AlreadyActivatedPlugin,
    StillActivatedPlugin,
    DeactivatedPlugin,
    ActivationFailed,
    PluginEntryNotFound,
    PluginNotFound,
    MissingPluginFactory,
    InstantiationFailed,
    ProcessingFailed,
    ProcessorHandlePoisoned,
    ProcessingStopped,
    ProcessingStarted,
    NullFactoryCreatePluginFunction,
    NullProcessFunction,
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
            Self::PluginEntryNotFound => write!(f, "No entry found for the specified plugin"),
            Self::PluginNotFound => write!(f, "Specified plugin was not found"),
            Self::MissingPluginFactory => write!(f, "No plugin factory was provided"),
            Self::InstantiationFailed => write!(f, "Could not instantiate"),
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

impl<H: for<'a> Host<'a>> From<ProcessingStartError<H>> for HostError {
    #[inline]
    fn from(_: ProcessingStartError<H>) -> Self {
        Self::StartProcessingFailed
    }
}
