use core::fmt::Display;
use std::error::Error;
use std::ffi::NulError;
use std::fmt::Formatter;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum LogError {
    NulError(NulError),
    FmtError(core::fmt::Error),
}

impl From<NulError> for LogError {
    #[inline]
    fn from(e: NulError) -> Self {
        LogError::NulError(e)
    }
}

impl From<core::fmt::Error> for LogError {
    #[inline]
    fn from(e: core::fmt::Error) -> Self {
        LogError::FmtError(e)
    }
}

impl Display for LogError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::NulError(e) => {
                write!(f, "Failed to encode message into a C String: {e}")
            }
            LogError::FmtError(e) => write!(f, "Message could not be formatted: {e}"),
        }
    }
}

impl Error for LogError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LogError::NulError(e) => Some(e),
            LogError::FmtError(e) => Some(e),
        }
    }
}
