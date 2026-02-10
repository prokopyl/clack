mod wrapper;

use clack_host::prelude::HostError;
pub use wrapper::*;

mod descriptor;
use crate::preset_discovery::preset_data::*;
pub(crate) use descriptor::*;

/// An indexer implementation.
///
/// It must be provided to a [preset finder provider instance](crate::preset_discovery::provider::ProviderInstance)
/// during initialization.
pub trait IndexerImpl: Sized {
    /// Declares a preset file type.
    ///
    /// # Errors
    /// This can return a [`HostError`] if the file type is invalid, or if any other error occurred.
    fn declare_filetype(&mut self, file_type: FileType) -> Result<(), HostError>;

    /// Declares a preset location for the host to index.
    ///
    /// # Errors
    /// This can return a [`HostError`] if the location is invalid, or if any other error occurred.
    fn declare_location(&mut self, location: LocationInfo) -> Result<(), HostError>;

    /// Declares a soundpack.
    ///
    /// # Errors
    /// This can return a [`HostError`] if the soundpack is invalid, or if any other error occurred.
    fn declare_soundpack(&mut self, soundpack: Soundpack) -> Result<(), HostError>;
}
