use core::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum HostError {
    StartProcessingFailed,
    AlreadyActivatedPlugin,
    DeactivatedPlugin,
    ActivationFailed,
    PluginEntryNotFound,
    PluginNotFound,
    MissingPluginFactory,
    InstantiationFailed,
    PluginIdNulError,
    ProcessingFailed,
    ProcessorHandlePoisoned,
    ProcessingStopped,
    ProcessingStarted,
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::StartProcessingFailed => write!(f, "Could not start processing"),
            Self::AlreadyActivatedPlugin => write!(f, "Plugin was already activated"),
            Self::DeactivatedPlugin => write!(f, "Plugin is currently deactivated"),
            Self::ActivationFailed => write!(f, "Unable to activate"),
            Self::PluginEntryNotFound => write!(f, "No entry found for the specified plugin"),
            Self::PluginNotFound => write!(f, "Specified plugin was not found"),
            Self::MissingPluginFactory => write!(f, "No plugin factory was provided"),
            Self::InstantiationFailed => write!(f, "Could not instantiate"),
            Self::PluginIdNulError => write!(f, "Plugin ID was null"),
            Self::ProcessingFailed => write!(f, "Could not process"),
            Self::ProcessorHandlePoisoned => write!(f, "Audio Processor handle was poisoned"),
            Self::ProcessingStopped => write!(f, "Audio Processor is currently stopped"),
            Self::ProcessingStarted => write!(f, "Audio Processor is currently started"),
        }
    }
}

impl std::error::Error for HostError {}
