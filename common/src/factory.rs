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
/// The role of this trait is to tie a Rust type to a standard CLAP factory identifier and its
/// matching raw C ABI type.
/// This is then used by the Clack APIs to always match the correct extension type from its
/// identifier.
///
/// This trait also defines how a factory pointer should be transformed to a reference to the
/// factory type.
///
/// # Safety
///
/// The [`IDENTIFIER`](Factory::IDENTIFIERS) **must** match the official identifier for the given
/// factory, otherwise the factory data could be misinterpreted, leading to Undefined Behavior.
pub unsafe trait Factory<'a>: Copy + Sized + Send + Sync {
    /// The standard identifier for this extension.
    const IDENTIFIERS: &'static [&'static CStr];
    type Raw: Copy + Sized + Send + Sync + 'static;

    /// Returns an instance of the extension from a given extension pointer.
    unsafe fn from_raw(raw: RawFactoryPointer<'a, Self::Raw>) -> Self;
}
