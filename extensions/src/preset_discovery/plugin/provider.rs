mod instance;
use crate::preset_discovery::plugin::metadata_receiver::MetadataReceiver;
use crate::preset_discovery::preset_data::Location;
pub use instance::ProviderInstance;

mod wrapper;

pub trait ProviderImpl<'a>: 'a {
    fn get_metadata(&mut self, location: Location, receiver: &mut MetadataReceiver);
}
