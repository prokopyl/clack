//! Factory types and associated utilities.
//!
//! In CLAP, factories are singleton objects exposed by the plugin bundle's
//! [entry point](crate::entry), which can in turn expose various functionalities.
//!
//! Each factory type has a standard, unique [identifier](Factory::IDENTIFIERS), which allows hosts
//! to query plugins for known factory type implementations.
//!
//! In Clack, factory implementations are represented by the [`Factory`] trait.
//!
//! The main factory type is the [`PluginFactory`](plugin::PluginFactory), which enables hosts to
//! list all the plugin implementations present in a bundle, and then instantiate on of them.
//!

use core::ffi::CStr;

mod raw;
pub use raw::RawFactoryPointer;

pub mod plugin;

/// A CLAP factory pointer.
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
///
/// # Example
///
/// This example is a shortened snippet of the implementation of the [`PluginFactory`](plugin::PluginFactory) type.
///
/// ```
/// use clack_common::factory::{Factory, RawFactoryPointer};
/// use clap_sys::factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
/// use core::ffi::CStr;
///
/// #[derive(Copy, Clone)]
/// pub struct PluginFactory<'a>(RawFactoryPointer<'a, clap_plugin_factory>);
///
/// // SAFETY: We have checked and ensured that CLAP_PLUGIN_FACTORY_ID is indeed the standard
/// // identifier for the clap_plugin_factory type.
/// unsafe impl<'a> Factory<'a> for PluginFactory<'a> {
///     const IDENTIFIERS: &'static [&'static CStr] = &[CLAP_PLUGIN_FACTORY_ID];
///     type Raw = clap_plugin_factory;
///
///     #[inline]
///     unsafe fn from_raw(raw: RawFactoryPointer<'a, Self::Raw>) -> Self {
///         Self(raw)
///     }
/// }
///
/// impl<'a> PluginFactory<'a> {
///     pub fn plugin_count(&self) -> u32 {
///         let Some(get_plugin_count) = self.0.get().get_plugin_count else {
///             return 0;
///         };
///
///         // SAFETY: this type can only get constructed from a plugin-provided pointer, so the
///         // CLAP spec enforces that this function pointer is actually valid to call.
///         unsafe { get_plugin_count(self.0.as_ptr()) }
///     }
/// }
///
/// ```
pub unsafe trait Factory<'a>: Copy + Sized + Send + Sync {
    /// The standard identifier for this extension.
    const IDENTIFIERS: &'static [&'static CStr];
    type Raw: Copy + Sized + Send + Sync + 'static;

    /// Returns an instance of the extension from a given extension pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that not only is the factory structure the `raw` pointer points
    /// to is valid, but also
    unsafe fn from_raw(raw: RawFactoryPointer<'a, Self::Raw>) -> Self;
}
