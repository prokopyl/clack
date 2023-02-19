//! Traits and associated utilities to handle and implement CLAP extensions.
//!
//! See the documentation of the `extensions` module in `clack-plugin` and `clack-host` for
//! implementation examples.

use core::ffi::c_void;
use std::ffi::CStr;
use std::ptr::NonNull;

/// A marker struct that represents extensions to be implemented by the plugin side.
///
/// See [`Extension::ExtensionSide`].
pub struct PluginExtensionSide;

/// A marker struct that represents extensions to be implemented by the host side.
///
/// See [`Extension::ExtensionSide`].
pub struct HostExtensionSide;

/// An extension side marker: either [`PluginExtensionSide`] or [`HostExtensionSide`].
///
/// See [`Extension::ExtensionSide`].
pub trait ExtensionSide: private::Sealed {}
impl ExtensionSide for PluginExtensionSide {}
impl ExtensionSide for HostExtensionSide {}

mod private {
    use super::*;

    pub trait Sealed {}
    impl Sealed for PluginExtensionSide {}
    impl Sealed for HostExtensionSide {}
}

/// A type representing a CLAP extension ABI.
///
/// The role of this trait is to tie a type to a standard CLAP extension identifier.
/// This is then used by some Clack methods to retrieve the correct extension type from its
/// identifier.
///
/// This trait also defines how an extension pointer should be transformed to a reference to the
/// extension type. By default a simple pointer cast is done.
///
/// # Safety
/// The [`IDENTIFIER`](Extension::IDENTIFIER) **must** match the official identifier for the given
/// extension, otherwise the extension data could be misinterpreted, leading to Undefined Behavior.
///
/// By default, the implementation of the [`Extension::from_extension_ptr`] simply casts the received pointer
/// to a shared reference to the Extension type. This implies the type implementing this trait
/// must be `#[repr(C)]` and ABI-compatible with the CLAP extension struct, unless the
/// [`Extension::from_extension_ptr`] method is overridden and implemented manually.
pub unsafe trait Extension: Sized + Send + Sync + 'static {
    /// The standard identifier for this extension.
    const IDENTIFIER: &'static CStr;
    /// Whether this is a host extension or a plugin extension
    type ExtensionSide: ExtensionSide;

    /// Returns an instance of the extension from a given extension pointer.
    ///
    /// The default implementation of this method simply casts the pointer.
    ///
    /// # Safety
    /// Callers must ensure the extension pointer points to the correct type, and also be valid for
    /// the duration of `'a`.
    #[inline]
    unsafe fn from_extension_ptr<'a>(ptr: NonNull<c_void>) -> &'a Self {
        ptr.cast().as_ref()
    }
}

/// Provides an implementation of this extension for a given type `I` (typically either a host or
/// plugin structure).
pub trait ExtensionImplementation<I>: Extension {
    /// The implementation of the extension.
    const IMPLEMENTATION: &'static Self;
}
