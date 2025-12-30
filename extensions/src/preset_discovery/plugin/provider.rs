mod instance;
use crate::preset_discovery::plugin::metadata_receiver::MetadataReceiver;
pub use instance::ProviderInstance;

mod wrapper;

pub trait Provider<'a>: 'a {
    fn get_metadata(&mut self, receiver: MetadataReceiver<'_>);
}
