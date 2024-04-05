//! Traits and associated utilities to handle and implement CLAP extensions.
//!
//! See the documentation of the `extensions` module in the `clack-plugin` and `clack-host` crates
//! for implementation examples.

use std::ffi::CStr;

mod raw;
pub use raw::{RawExtension, RawExtensionImplementation};

/// A marker struct that represents extensions to be implemented by the plugin side.
///
/// See [`Extension::ExtensionSide`].
#[derive(Copy, Clone)]
pub struct PluginExtensionSide;

/// A marker struct that represents extensions to be implemented by the host side.
///
/// See [`Extension::ExtensionSide`].
#[derive(Copy, Clone)]
pub struct HostExtensionSide;

/// An extension side marker: either [`PluginExtensionSide`] or [`HostExtensionSide`].
///
/// See [`Extension::ExtensionSide`].
pub trait ExtensionSide: private::Sealed + Copy + Sized {}
impl ExtensionSide for PluginExtensionSide {}
impl ExtensionSide for HostExtensionSide {}

mod private {
    use super::*;

    pub trait Sealed {}
    impl Sealed for PluginExtensionSide {}
    impl Sealed for HostExtensionSide {}
}

/// A type representing a CLAP extension pointer.
///
/// The role of this trait is to tie a type to a standard CLAP extension identifier.
/// This is then used by the Clack APIs to always match the correct extension type from its
/// identifier.
///
/// This trait also defines how an extension pointer should be transformed to a reference to the
/// extension type.
///
/// # Safety
/// The [`IDENTIFIER`](Extension::IDENTIFIER) **must** match the official identifier for the given
/// extension, otherwise the extension data could be misinterpreted, leading to Undefined Behavior.
///
/// By default, the implementation of the [`Extension::from_raw`] simply casts the received pointer
/// to a shared reference to the Extension type. This implies the type implementing this trait
/// must be `#[repr(C)]` and ABI-compatible with the CLAP extension struct, unless the
/// [`Extension::from_raw`] method is overridden and implemented manually.
pub unsafe trait Extension: Copy + Sized + Send + Sync + 'static {
    /// The standard identifier for this extension.
    const IDENTIFIER: &'static CStr;
    /// Whether this is a host extension or a plugin extension
    type ExtensionSide: ExtensionSide;

    /// Returns an instance of the extension from a given extension pointer.
    ///
    /// # Safety
    /// Callers must ensure the extension pointer points to the extension type that matches
    /// [`Self::IDENTIFIER`].
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self;
}

/// Provides an implementation of this extension for a given type `I` (typically either a host or
/// plugin structure).
pub trait ExtensionImplementation<I>: Extension {
    /// The implementation of the extension.
    const IMPLEMENTATION: RawExtensionImplementation;
}
