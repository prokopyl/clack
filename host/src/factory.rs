#![deny(missing_docs)]

//! Factory types and associated utilities.
//!
//! In CLAP, factories are singleton objects exposed by the [plugin bundle](crate::bundle)'s
//! entry point, which can in turn expose various functionalities.
//!
//! Each factory type has a standard, unique [identifier](Factory::IDENTIFIER), which allows hosts
//! to query plugins for known factory type implementations.
//!
//! In Clack, factories are represented by the [`Factory`] trait.
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

use crate::host::HostError;
pub use clack_common::factory::*;
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use std::ffi::CStr;
use std::ptr::NonNull;

mod plugin_descriptor;
pub use plugin_descriptor::*;

/// A [`Factory`] that exposes a list of [`PluginDescriptor`s](PluginDescriptor).
///
/// # Example
///
///```
/// use clack_host::prelude::PluginBundle;
///
/// # mod diva { include!("./bundle/diva_stub.rs"); }
/// # let bundle = unsafe { PluginBundle::load_from_raw(&diva::DIVA_STUB_ENTRY, "/home/user/.clap/u-he/libdiva.so").unwrap() };
/// # #[cfg(never)]
/// let bundle = PluginBundle::load("/home/user/.clap/u-he/libdiva.so")?;
///
/// // Fetch the PluginFactory from the bundle, if present
/// let plugin_factory = bundle.get_plugin_factory().unwrap();
///
/// println!("The bundle exposes {} plugins:", plugin_factory.plugin_count());
///
/// for plugin_descriptor in plugin_factory {
///    println!("- {}", plugin_descriptor.name().unwrap().to_string_lossy());
///    println!("\t ID: {}", plugin_descriptor.id().unwrap().to_string_lossy());
///    
///    let features: Vec<_> = plugin_descriptor.features().map(|f| f.to_string_lossy()).collect();
///    println!("\t Features: [{}]", features.join(", "));
/// }
/// ```
#[repr(C)]
pub struct PluginFactory {
    inner: clap_plugin_factory,
}

unsafe impl Factory for PluginFactory {
    const IDENTIFIER: &'static CStr = CLAP_PLUGIN_FACTORY_ID;
}

impl PluginFactory {
    /// Returns the number of plugin descriptors exposed by this plugin factory.
    #[inline]
    pub fn plugin_count(&self) -> u32 {
        // SAFETY: no special safety considerations
        match self.inner.get_plugin_count {
            None => 0,
            Some(count) => unsafe { count(&self.inner) },
        }
    }

    /// Returns an the [`PluginDescriptor`s](PluginDescriptor) exposed by this plugin
    /// factory at a given index, or `None` if there is no plugin descriptor at the given index.
    ///
    /// Implementations on the plugin-side *should* return a descriptor for any index strictly less
    /// than [`plugin_count`](PluginFactory::plugin_count), but this is not a guarantee.
    ///
    /// See also the [`plugin_descriptors`](PluginFactory::plugin_descriptors) method for a
    /// convenient iterator of all the plugin descriptors exposed by this factory.
    #[inline]
    pub fn plugin_descriptor(&self, index: u32) -> Option<PluginDescriptor> {
        // SAFETY: descriptor is guaranteed not to outlive the entry
        unsafe { (self.inner.get_plugin_descriptor?)(&self.inner, index).as_ref() }
            .map(PluginDescriptor::from_raw)
    }

    /// Returns an iterator of all the [`PluginDescriptor`s](PluginDescriptor) exposed by this
    /// plugin factory.
    ///
    /// For convenience, the [`&PluginFactory`](PluginFactory) type implements the
    /// [`IntoIterator`] trait, which also returns this iterator.
    ///
    /// See also the [`plugin_descriptor`](PluginFactory::plugin_descriptor) method to retrieve
    /// a plugin descriptor at a specific index.
    #[inline]
    pub fn plugin_descriptors(&self) -> PluginDescriptorsIter {
        PluginDescriptorsIter {
            factory: self,
            count: self.plugin_count(),
            current_index: 0,
        }
    }

    pub(crate) unsafe fn create_plugin(
        &self,
        plugin_id: &CStr,
        host: *const clap_host,
    ) -> Result<NonNull<clap_plugin>, HostError> {
        let plugin = NonNull::new((self
            .inner
            .create_plugin
            .ok_or(HostError::NullFactoryCreatePluginFunction)?)(
            &self.inner,
            host,
            plugin_id.as_ptr(),
        ) as *mut clap_plugin)
        .ok_or(HostError::PluginNotFound)?;

        if let Some(init) = plugin.as_ref().init {
            if !init(plugin.as_ptr()) {
                if let Some(destroy) = plugin.as_ref().destroy {
                    destroy(plugin.as_ptr());
                }

                return Err(HostError::InstantiationFailed);
            }
        }

        Ok(plugin)
    }
}

/// An [`Iterator`] over all the [`PluginDescriptor`s](PluginDescriptor) exposed by a
/// plugin factory.
///
/// See the [`PluginFactory::plugin_descriptors`] method that produces this iterator.
pub struct PluginDescriptorsIter<'a> {
    factory: &'a PluginFactory,
    current_index: u32,
    count: u32,
}

impl<'a> Iterator for PluginDescriptorsIter<'a> {
    type Item = PluginDescriptor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_index >= self.count {
                return None;
            }

            let descriptor = self.factory.plugin_descriptor(self.current_index);
            self.current_index += 1;

            // Skip all none-returning indexes
            if let Some(d) = descriptor {
                return Some(d);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count as usize, Some(self.count as usize))
    }
}

/// Returns an iterator of all the [`PluginDescriptor`s](PluginDescriptor) exposed by this plugin
/// factory.
impl<'a> IntoIterator for &'a PluginFactory {
    type Item = PluginDescriptor<'a>;
    type IntoIter = PluginDescriptorsIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.plugin_descriptors()
    }
}
