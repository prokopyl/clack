mod instance;
pub use instance::ProviderInstance;

mod wrapper;

pub trait Provider<'a>: 'a {}
