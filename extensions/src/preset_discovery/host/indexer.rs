mod wrapper;
pub use wrapper::*;

mod descriptor;
use crate::preset_discovery::*;
pub(crate) use descriptor::*;

pub trait Indexer: Sized {
    // TODO: errors
    fn declare_filetype(&mut self, file_type: FileType);
    fn declare_location(&mut self, location: LocationData);
    fn declare_soundpack(&mut self, soundpack: Soundpack);
}
