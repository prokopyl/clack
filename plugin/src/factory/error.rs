use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum FactoryError {
    NullFactoryInstance,
    NulPtr(&'static str),
    Panic,
}

impl Display for FactoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FactoryError::NullFactoryInstance => f.write_str(
                "Plugin factory method was called with null clap_plugin_factory pointer",
            ),
            FactoryError::NulPtr(ptr_name) => {
                write!(
                    f,
                    "Plugin factory method was called with null {ptr_name} pointer"
                )
            }
            FactoryError::Panic => f.write_str("Plugin factory panicked"),
        }
    }
}

impl Error for FactoryError {}
