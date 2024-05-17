use core::fmt;
use core::fmt::{Debug, Display, Formatter};
use std::error::Error;

/// A generic, type-erased error type for host-originating errors.
///
/// Errors are type-erased because the CLAP API does not support extracting error information from
/// a plugin or host, only that an error happened.
///
/// Errors originating from a user-provided host callback implementation are simply logged through
/// the host's provided logging facilities if available, or the standard error output ([`stderr`])
/// if not.
///
/// This error can be constructed either from any existing [`Error`] type, or from an arbitrary
/// error message.
///
/// # Example
///
/// ```
/// use std::io;
/// use clack_host::prelude::HostError;
///
/// fn foo () -> io::Result<()> {
///     /* ... */
///     # Ok(())
/// }
///
/// fn perform(valid: bool) -> Result<(), HostError> {
///     if !valid {
///         return Err(HostError::Message("Invalid value"))
///     }
///     /* ... */
///     foo()?;
///     /* ... */
///     Ok(())
/// }
/// # perform(true).unwrap()
/// ```
///
/// [`stderr`]: std::io::stderr
#[derive(Debug)]
pub enum HostError {
    /// A generic, type-erased error.
    Error(Box<dyn Error + 'static>),
    /// A constant string message to be displayed.
    Message(&'static str),
}

impl Display for HostError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HostError::Error(e) => Display::fmt(&e, f),
            HostError::Message(msg) => f.write_str(msg),
        }
    }
}

impl<E: Error + 'static> From<E> for HostError {
    #[inline]
    fn from(e: E) -> Self {
        HostError::Error(Box::new(e))
    }
}
