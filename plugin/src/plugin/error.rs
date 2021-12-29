use clap_sys::ext::log::{
    clap_log_severity, CLAP_LOG_ERROR, CLAP_LOG_HOST_MISBEHAVING, CLAP_LOG_PLUGIN_MISBEHAVING,
};
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

#[derive(Debug)]
pub enum PluginInternalError<E: Error = PluginError> {
    NulPluginDesc,
    NulPluginData,
    UninitializedPlugin,
    ActivatedPlugin,
    DeactivatedPlugin,
    Panic,
    Other(E),
}

impl<E: Error> PluginInternalError<E> {
    pub fn severity(&self) -> clap_log_severity {
        match self {
            PluginInternalError::Other(_) => CLAP_LOG_ERROR,
            PluginInternalError::Panic => CLAP_LOG_PLUGIN_MISBEHAVING,
            _ => CLAP_LOG_HOST_MISBEHAVING,
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
            PluginInternalError::ActivatedPlugin => f.write_str("Plugin was already activated"),
            PluginInternalError::DeactivatedPlugin => {
                f.write_str("Plugin was not activated before calling a processing-thread method")
            }
            PluginInternalError::Other(e) => std::fmt::Display::fmt(&e, f),
            PluginInternalError::Panic => f.write_str("Plugin panicked"), // TODO: stacktrace
        }
    }
}

impl<E: Error> Error for PluginInternalError<E> {}
