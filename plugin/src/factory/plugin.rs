//! The plugin factory type.
//!
//! In CLAP, the Plugin Factory is the main factory type (and at the time of writing, the only
//! stable standard one). Its purpose is to expose to the host a list of all the plugin types
//! included in this bundle, and to allow the host to instantiate them.
//!
//! See the
//!
//! See the [`factory` module documentation](crate::factory) to learn more about factories.

use crate::factory::Factory;
use crate::host::HostInfo;
use crate::plugin::{PluginDescriptor, PluginInstance};
use clap_sys::factory::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use clap_sys::host::clap_host;
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use std::ffi::CStr;

/// A wrapper around a given [`PluginFactory`] implementation.
///
/// This wrapper is required in order to expose a C FFI-compatible factory to the host, and is what
/// needs to be exposed by an [`Entry`](crate::entry::Entry).
#[repr(C)]
pub struct PluginFactoryWrapper<F> {
    raw: clap_plugin_factory,
    factory: F,
}

impl<F: PluginFactory> PluginFactoryWrapper<F> {
    /// Wraps a given [`PluginFactory`] instance.
    pub const fn new(factory: F) -> Self {
        Self {
            raw: clap_plugin_factory {
                get_plugin_count: Some(Self::get_plugin_count),
                get_plugin_descriptor: Some(Self::get_plugin_descriptor),
                create_plugin: Some(Self::create_plugin),
            },
            factory,
        }
    }

    /// Returns a shared reference to the wrapped [`PluginFactory`].
    #[inline]
    pub fn factory(&self) -> &F {
        &self.factory
    }

    /// Returns a raw CLAP plugin factory pointer, ready to be used by the host.
    #[inline]
    pub fn as_raw_ptr(&self) -> *const clap_plugin_factory {
        &self.raw
    }

    unsafe extern "C" fn get_plugin_count(factory: *const clap_plugin_factory) -> u32 {
        let this = &*(factory as *const Self);
        this.factory.plugin_count()
    }

    unsafe extern "C" fn get_plugin_descriptor(
        factory: *const clap_plugin_factory,
        index: u32,
    ) -> *const clap_plugin_descriptor {
        let this = &*(factory as *const Self);

        match this.factory.plugin_descriptor(index) {
            None => core::ptr::null(),
            Some(d) => d.as_raw(),
        }
    }

    unsafe extern "C" fn create_plugin(
        factory: *const clap_plugin_factory,
        clap_host: *const clap_host,
        plugin_id: *const std::os::raw::c_char,
    ) -> *const clap_plugin {
        let plugin_id = CStr::from_ptr(plugin_id);
        if clap_host.is_null() {
            eprintln!("[ERROR] Null clap_host pointer was provided to entry::create_plugin.");
            return core::ptr::null();
        };

        let host_info = HostInfo::from_raw(clap_host);
        let this = &*(factory as *const Self);

        match this.factory.instantiate_plugin(host_info, plugin_id) {
            None => core::ptr::null(),
            Some(instance) => instance.into_owned_ptr(),
        }
    }
}

unsafe impl<F> Factory for PluginFactoryWrapper<F> {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_ID;
}

/// A Plugin Factory implementation.
///
/// See the [module documentation](self) to learn more about the role of a Plugin Factory.
///
/// # Example
///
/// The following example shows how to implement a basic, single-plugin factory.
///
/// ```
/// use std::ffi::CStr;
/// use clack_plugin::entry::prelude::*;
/// use clack_plugin::prelude::*;
///
/// pub struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     type AudioProcessor<'a> = ();
///     type Shared<'a> = ();
///     type MainThread<'a> = ();
/// }
///
/// pub struct MyPluginFactory {
///     plugin_descriptor: PluginDescriptor
/// }
///
/// impl PluginFactory for MyPluginFactory {
///     fn plugin_count(&self) -> u32 {
///         1 // We only have a single plugin
///     }
///
///     fn plugin_descriptor(&self, index: u32) -> Option<&PluginDescriptor> {
///         match index {
///             0 => Some(&self.plugin_descriptor),
///             _ => None
///         }
///     }
///
///     fn instantiate_plugin<'a>(&'a self, host_info: HostInfo<'a>, plugin_id: &CStr) -> Option<PluginInstance<'a>> {
///         if plugin_id == self.plugin_descriptor.id() {
///             Some(PluginInstance::new::<MyPlugin>(
///                 host_info,
///                 &self.plugin_descriptor,
///                 |_host| Ok(()) /* Create the shared struct */,
///                 |_host, _shared| Ok(()) /* Create the main thread struct */,
///             ))
///         } else {
///             None
///         }
///     }
/// }
/// ```
pub trait PluginFactory: Send + Sync {
    /// Returns the number of plugins exposed by this factory.
    fn plugin_count(&self) -> u32;

    /// Returns the [`PluginDescriptor`] of the plugin that is assigned the given index.
    ///
    /// Hosts will usually call this method repeatedly with every index from 0 to the total returned
    /// by [`plugin_count`](PluginFactory::plugin_count), in order to discover all the plugins
    /// exposed by this factory.
    ///
    /// If the given index is out of bounds, or in general does not match any given plugin, this
    /// returns [`None`].
    fn plugin_descriptor(&self, index: u32) -> Option<&PluginDescriptor>;

    /// Creates a new plugin instance for the plugin type matching the given `plugin_id`.
    ///
    /// If the given `plugin_id` matches against one of the plugin this factory manages,
    /// implementors of this trait then use the [`PluginInstance::new`] method to instantiate the
    /// corresponding plugin implementation.
    ///
    /// If the given `plugin_id` does not match any known plugins to this factory, this method
    /// returns [`None`].
    fn instantiate_plugin<'a>(
        &'a self,
        host_info: HostInfo<'a>,
        plugin_id: &CStr,
    ) -> Option<PluginInstance<'a>>;
}
