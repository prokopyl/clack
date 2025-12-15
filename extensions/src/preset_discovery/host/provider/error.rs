use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub enum ProviderInstanceError {
    MissingPresetDiscoveryFactory,
    NullFactoryCreateFunction,
    NullInitFunction,
    CreationFailed,
    InitFailed,
}

impl ProviderInstanceError {
    fn msg(&self) -> &'static str {
        match self {
            ProviderInstanceError::MissingPresetDiscoveryFactory => {
                "Bundle does not expose a Preset Discovery Factory"
            }
            _ => todo!(),
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
