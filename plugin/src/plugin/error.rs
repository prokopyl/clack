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
