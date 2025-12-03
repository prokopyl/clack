#![deny(missing_docs)]

use clap_sys::ext::log::{
    CLAP_LOG_ERROR, CLAP_LOG_HOST_MISBEHAVING, CLAP_LOG_PLUGIN_MISBEHAVING, clap_log_severity,
};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors raised by a [`FactoryWrapper`](super::wrapper::FactoryWrapper).
#[derive(Debug)]
#[non_exhaustive]
pub enum FactoryWrapperError {
    /// The factory raw pointer was null.
    NullFactoryInstance,
    /// An unexpectedly null raw pointer was encountered.
    ///
    /// The given string may contain more information about which pointer was found to be null.
    NulPtr(&'static str),
    /// The plugin factory panicked during a function call.
    Panic,
    /// A generic or custom error of a given severity.
    Error(clap_log_severity, Box<dyn Error>),
}

impl FactoryWrapperError {
    /// Returns the severity of this error.
    ///
    /// This is mainly useful for logging.
    ///
    /// # Example
    ///
    /// ```
    /// use clap_sys::ext::log::CLAP_LOG_PLUGIN_MISBEHAVING;
    /// use clack_plugin::extensions::wrapper::PluginWrapperError;
    /// let error = PluginWrapperError::Panic;
    ///
    /// assert_eq!(error.severity(), CLAP_LOG_PLUGIN_MISBEHAVING);
    /// ```
    pub fn severity(&self) -> clap_log_severity {
        match self {
            FactoryWrapperError::Panic => CLAP_LOG_PLUGIN_MISBEHAVING,
            FactoryWrapperError::Error(s, _) => *s,
            _ => CLAP_LOG_HOST_MISBEHAVING,
        }
    }

    /// Returns a closure that maps an error to a [`FactoryWrapperError::Error`] error of a given
    /// severity.
    ///
    /// This is a useful helper method when paired with [`Result::map_err`].
    ///
    /// # Example
    /// ```
    /// use clap_sys::ext::log::CLAP_LOG_PLUGIN_MISBEHAVING;
    /// use clack_plugin::extensions::wrapper::PluginWrapperError;
    ///
    /// let x: Result<(), _> = Err(std::env::VarError::NotPresent); // Some random error type
    /// let clap_error = x.map_err(PluginWrapperError::with_severity(CLAP_LOG_PLUGIN_MISBEHAVING));
    ///
    /// assert_eq!(clap_error.unwrap_err().severity(), CLAP_LOG_PLUGIN_MISBEHAVING);
    /// ```
    #[inline]
    pub fn with_severity<E: 'static + Error>(
        severity: clap_log_severity,
    ) -> impl Fn(E) -> FactoryWrapperError {
        move |e| FactoryWrapperError::Error(severity, Box::new(e))
    }
}

impl Display for FactoryWrapperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use FactoryWrapperError::*;
        match self {
            NullFactoryInstance => f.write_str(
                "Plugin factory method was called with null clap_plugin_factory pointer",
            ),
            NulPtr(ptr_name) => {
                write!(
                    f,
                    "Plugin factory method was called with null {ptr_name} pointer"
                )
            }
            Error(_, e) => std::fmt::Display::fmt(e, f),
            Panic => f.write_str("Plugin factory panicked"),
        }
    }
}

impl<E: Error + 'static> From<E> for FactoryWrapperError {
    fn from(value: E) -> Self {
        Self::Error(CLAP_LOG_ERROR, Box::new(value))
    }
}
