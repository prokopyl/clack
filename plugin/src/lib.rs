#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/prokopyl/clack/main/logo.svg")]

pub mod entry;
pub mod extension;
pub mod host;
pub mod plugin;
pub mod process;

pub use clack_common as common;
