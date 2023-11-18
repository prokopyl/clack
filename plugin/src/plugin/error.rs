use crate::process::audio::BufferError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum PluginError {
    AlreadyActivated,
    OperationFailed,
    AudioBufferError(BufferError),
    Io(std::io::Error),
    Custom(Box<dyn Error + 'static>),
    Message(&'static str),
}

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::AlreadyActivated => {
                write!(
                    f,
                    "This plugin's activate() function was called while already activated"
                )
            }
            PluginError::OperationFailed => write!(f, "The requested operation has failed"),
            PluginError::Custom(e) => std::fmt::Display::fmt(&e, f),
            PluginError::Io(e) => std::fmt::Display::fmt(&e, f),
            PluginError::AudioBufferError(e) => std::fmt::Display::fmt(&e, f),
            PluginError::Message(msg) => f.write_str(msg),
        }
    }
}

impl Error for PluginError {}

impl From<std::io::Error> for PluginError {
    #[inline]
    fn from(e: std::io::Error) -> Self {
        PluginError::Io(e)
    }
}

impl From<BufferError> for PluginError {
    #[inline]
    fn from(e: BufferError) -> Self {
        PluginError::AudioBufferError(e)
    }
}
