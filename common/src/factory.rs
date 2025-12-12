//! Traits and associated utilities to handle and implement CLAP factories.
//!
//! See the documentation of the `factory` module in the `clack-plugin` and `clack-host` crates
//! for implementation examples.

use core::ffi::CStr;

mod raw;
pub use raw::RawFactoryPointer;

mod plugin;
pub use plugin::PluginFactory;

/// A type representing a CLAP factory pointer.
///
/// The role of this trait is to tie a raw type to a standard CLAP factory identifier.
/// This is then used by the Clack APIs to always match the correct extension type from its
/// identifier.
///
/// This trait also defines how an extension pointer should be transformed to a reference to the
/// extension type.
///
/// # Safety
///
/// The [`IDENTIFIER`](Extension::IDENTIFIERS) **must** match the official identifier for the given
/// extension, otherwise the extension data could be misinterpreted, leading to Undefined Behavior.
pub unsafe trait Factory<'a>: Copy + Sized + Send + Sync {
    /// The standard identifier for this extension.
    const IDENTIFIERS: &'static [&'static CStr];
    type Raw: Copy + Sized + Send + Sync + 'static;

    /// Returns an instance of the extension from a given extension pointer.
    fn from_raw(raw: RawFactoryPointer<'a, Self::Raw>) -> Self;
}
