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
//! [`PluginFactory`](plugin::PluginFactoryImpl), which enables hosts to list all the plugin
//! implementations present in a bundle.
//!
//! See the [`Entry`](crate::entry::Entry) trait documentation for an example on how to create a
//! custom entry and plugin factory.

mod error;
pub mod plugin;
mod wrapper;

pub use clack_common::factory::*;
pub use error::FactoryWrapperError;
pub use wrapper::FactoryWrapper;

/// Provides an implementation of this extension for a given type `I` (typically either a host or
/// plugin structure).
pub trait FactoryImplementation {
    type Factory<'a>: Factory<'a>
    where
        Self: 'a;
    type Wrapped;

    fn wrapper(&self) -> &FactoryWrapper<<Self::Factory<'_> as Factory<'_>>::Raw, Self::Wrapped>;
}
