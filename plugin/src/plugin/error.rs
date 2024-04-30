use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum PluginError {
    Error(Box<dyn Error + 'static>),
    Message(&'static str),
}

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::Error(e) => std::fmt::Display::fmt(&e, f),
            PluginError::Message(msg) => f.write_str(msg),
        }
    }
}

impl<E: Error + 'static> From<E> for PluginError {
    #[inline]
    fn from(e: E) -> Self {
        PluginError::Error(Box::new(e))
    }
}
