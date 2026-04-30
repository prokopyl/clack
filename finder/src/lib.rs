#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]

mod finder;
mod paths;

pub use finder::{ClapFinder, ClapFinderIter, PotentialClapFile};
pub use paths::standard_clap_paths;
