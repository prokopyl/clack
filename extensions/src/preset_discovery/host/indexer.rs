mod wrapper;
pub use wrapper::*;

mod descriptor;
pub(crate) use descriptor::*;

pub trait Indexer: Sized {}
