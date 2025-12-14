#![deny(missing_docs)]

//! Factory types and associated utilities.
//!
//! In CLAP, factories are singleton objects exposed by the [plugin bundle](crate::bundle)'s
//! entry point, which can in turn expose various functionalities.
//!
//! Each factory type has a standard, unique [identifier](FactoryPointer::IDENTIFIER), which allows hosts
//! to query plugins for known factory type implementations.
//!
//! In Clack, pointers to factories are represented by the [`FactoryPointer`] trait.
//!
//! See the [`PluginBundle::get_factory`](crate::bundle::PluginBundle::get_factory) method to
//! retrieve a given factory type from a plugin bundle.
//!
//! The main factory type (and, at the time of this writing, the only stable standard one), is the
//! [`PluginFactory`], which enables hosts to list all the plugin implementations present in this
//! bundle using their [`PluginDescriptor`].
//!
//! See the [`PluginFactory`]'s type documentation for more detail and examples on how to
//! list plugins.

pub use clack_common::factory::*;
