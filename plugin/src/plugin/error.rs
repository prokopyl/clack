use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum PluginError {
    CannotRescale,
    AlreadyActivated,
    Io(std::io::Error),
    Custom(Box<dyn Error + 'static>),
}

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::CannotRescale => write!(f, "This plugin cannot be rescaled."),
            PluginError::AlreadyActivated => {
                write!(
                    f,
                    "This plugin's activate() function was called while already activated"
                )
            }
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
