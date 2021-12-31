use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub type Result<T = ()> = ::core::result::Result<T, PluginError>;

#[derive(Debug)]
pub enum PluginError {
    Io(std::io::Error),
    Custom(Box<dyn Error + 'static>),
}

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::Custom(e) => std::fmt::Display::fmt(&e, f),
            PluginError::Io(e) => std::fmt::Display::fmt(&e, f),
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
