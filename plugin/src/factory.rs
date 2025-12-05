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
//! The main factory type (and, at the time of this writing, the only stable standard one), is the
//! [`PluginFactory`](plugin::PluginFactory), which enables hosts to list all the plugin
//! implementations present in a bundle.
//!
//! See the [`Entry`](crate::entry::Entry) trait documentation for an example on how to create a
//! custom entry and plugin factory.

use core::ffi::c_void;
use std::ffi::CStr;
use std::ptr::NonNull;

mod error;
pub mod plugin;
mod wrapper;

pub use error::FactoryWrapperError;
pub use wrapper::FactoryWrapper;

/// A base trait for plugin-side factory implementations.
///
/// # Safety
///
/// Types implementing this trait and using the default implementation of
/// [`get_raw_factory_ptr`](Factory::get_raw_factory_ptr)
/// **MUST** be `#[repr(C)]` and have the same C-FFI representation as the CLAP factory struct
/// matching the factory's [`IDENTIFIER`](Factory::IDENTIFIERS).
///
/// Failure to do so will result in incorrect pointer casts and UB.
///
/// # Example
///
/// This example shows how to implement the [`Factory`] trait for a custom Plugin Factory type,
/// wrapping the raw type from `clap-sys`.
///
/// Values of this custom factory type will then be able to be used in the
/// [`EntryFactories::register_factory`](crate::entry::EntryFactories::register_factory) method,
/// which will make them available to the host.
///
/// ```
/// use clack_plugin::factory::Factory;
/// use clap_sys::factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
/// use std::ffi::CStr;
///
/// #[repr(C)]
/// pub struct MyPluginFactory(clap_plugin_factory);
///
/// unsafe impl Factory for MyPluginFactory {
///     const IDENTIFIERS: &[&CStr] = &[CLAP_PLUGIN_FACTORY_ID];
/// }
/// ```
pub unsafe trait Factory {
    /// The standard identifier for this factory.
    const IDENTIFIERS: &[&CStr];

    /// Returns this factory as a C FFI-compatible raw pointer.
    ///
    /// The default implementation simply casts the given reference.
    #[inline]
    fn get_raw_factory_ptr(&self) -> NonNull<c_void> {
        NonNull::from(self).cast()
    }
}
