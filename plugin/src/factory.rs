//! Factory types and associated utilities.
//!
//! In CLAP, factories are singleton objects exposed by the plugin library's
//! [entry point](crate::entry), which can in turn expose various functionalities.
//!
//! Each factory type has a standard, unique [identifier](Factory::IDENTIFIERS), which allows hosts
//! to query plugins for known factory type implementations.
//!
//! In Clack, factory implementations are represented by the [`Factory`] trait.
//!
//! The main factory type (and, at the time of this writing, the only stable standard one), is the
//! [`PluginFactory`](plugin::PluginFactoryImpl), which enables hosts to list all the plugin
//! implementations present in an entry.
//!
//! See the [`Entry`](crate::entry::Entry) trait documentation for an example on how to create a
//! custom entry and plugin factory.

mod error;
pub mod plugin;
mod wrapper;

pub use clack_common::factory::*;
pub use error::FactoryWrapperError;
pub use wrapper::FactoryWrapper;

/// A types that provides a specific factory implementation.
///
/// # Safety
///
/// The wrapper returned by the [`wrapper`](Self::wrapper) function *must* wrap a `Raw` implementation
/// that fully complies to the CLAP specification of the given [`Factory`] type, and must remain
/// valid for the duration of the `'a` lifetime.
pub unsafe trait FactoryImplementation<'a>: 'a {
    /// The [`Factory`] type this is able to provide.
    type Factory: Factory<'a>;
    /// The inner factory implementation type in the [`FactoryWrapper`] returned by [`wrapper`](Self::wrapper)
    type Wrapped;

    /// Returns the [`FactoryWrapper`] instance this type is able to provide.
    fn wrapper(&self) -> &FactoryWrapper<<Self::Factory as Factory<'a>>::Raw, Self::Wrapped>;
}
