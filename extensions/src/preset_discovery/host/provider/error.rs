use std::error::Error;
use std::fmt::{Display, Formatter};

/// Errors that can occur while creating a [`Provider`](super::Provider).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ProviderInstanceError {
    /// The bundle does not actually provide a preset-discovery factory.
    MissingPresetDiscoveryFactory,
    /// The 'create' function pointer on the preset-discovery factory was NULL.
    NullFactoryCreateFunction,
    /// The 'init' function pointer on the returned provider instance was NULL.
    NullInitFunction,
    /// The 'create' function of the factory returned an error.
    CreationFailed,
    /// The 'init' function of the provider instance returned an error.
    InitFailed,
}

impl ProviderInstanceError {
    fn msg(&self) -> &'static str {
        use ProviderInstanceError::*;

        match self {
            MissingPresetDiscoveryFactory => "Bundle does not expose a Preset Discovery Factory",
            NullFactoryCreateFunction => "Preset factory 'create' function is NULL",
            NullInitFunction => "Provider 'init' function is NULL",
            CreationFailed => "Provider 'create' function failed",
            InitFailed => "Provider 'init' function failed",
        }
    }
}

impl Display for ProviderInstanceError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg())
    }
}

impl Error for ProviderInstanceError {}
