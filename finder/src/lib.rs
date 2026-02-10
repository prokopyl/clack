#![doc = include_str!("../README.md")]
#![deny(missing_docs, unsafe_code)]

mod finder;
mod paths;

pub use finder::{ClapBundle, ClapFinder, ClapFinderIter};
pub use paths::standard_clap_paths;
