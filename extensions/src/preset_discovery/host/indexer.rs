mod wrapper;
pub use wrapper::*;

mod descriptor;
use crate::preset_discovery::preset_data::*;
pub(crate) use descriptor::*;

pub trait IndexerImpl: Sized {
    // TODO: errors
    fn declare_filetype(&mut self, file_type: FileType);
    fn declare_location(&mut self, location: LocationInfo);
    fn declare_soundpack(&mut self, soundpack: Soundpack);
}
