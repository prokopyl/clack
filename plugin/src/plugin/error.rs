use clap_audio_common::extensions::log::LogSeverity;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub type Result<T = ()> = ::core::result::Result<T, PluginError>;

#[derive(Debug)]
pub enum PluginError {
    Custom(Box<dyn Error + 'static>),
}

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::Custom(e) => e.fmt(f),
        }
    }
}

impl Error for PluginError {}

pub enum PluginInternalError<E: Error> {
    NulPluginDesc,
    NulPluginData,
    UninitializedPlugin,
    Panic,
    Other(E),
}

impl<E: Error> PluginInternalError<E> {
    pub fn severity(&self) -> LogSeverity {
        match self {
            PluginInternalError::Other(_) => LogSeverity::Error,
            PluginInternalError::Panic => LogSeverity::PluginMisbehaving,
            _ => LogSeverity::HostMisbehaving,
        }
    }
}

impl<E: Error> From<E> for PluginInternalError<E> {
    #[inline]
    fn from(e: E) -> Self {
        PluginInternalError::Other(e)
    }
}

impl<E: Error> Display for PluginInternalError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginInternalError::NulPluginDesc => {
                f.write_str("Plugin method was called with null clap_plugin pointer")
            }
            PluginInternalError::NulPluginData => {
                f.write_str("Plugin method was called with null clap_plugin.plugin_data pointer")
            }
            PluginInternalError::UninitializedPlugin => {
                f.write_str("Plugin was not properly initialized before use")
            }
            PluginInternalError::Other(e) => std::fmt::Display::fmt(&e, f),
            PluginInternalError::Panic => f.write_str("Plugin panicked"), // TODO: stacktrace
        }
    }
}
