use crate::entry::prelude::*;
use crate::prelude::*;
use std::ffi::CStr;
use std::marker::PhantomData;

/// An [`Entry`] that only exposes a single plugin type to the host.
///
/// This is a simplified entry type, which only requires the simple [`SimplePlugin`] trait to be
/// implemented by the user, and implements an entry and plugin factory around it.
///
/// This entry type exists purely for convenience of the users in the common case of having a single
/// plugin type to expose to the host.
///
/// If you actually need to expose more plugin types, or to customize the entry's behavior in some
/// other way, see the [`Entry`] trait documentation for an example on how to implement your own
/// custom entry.
///
/// # Example
///
/// ```
/// use clack_plugin::entry::SimplePlugin;
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
/// impl SimplePlugin for MyPlugin {
///     fn get_descriptor() -> PluginDescriptor {
///         PluginDescriptor::new("my.plugin", "My Plugin")
///     }
///
///     fn new_shared<'a>(
///         _host: HostHandle<'a>
///     ) -> Result<Self::Shared<'a>, PluginError> {
///         Ok(())
///     }
///
///     fn new_main_thread<'a>(
///         host: HostMainThreadHandle<'a>,
///         shared: &'a Self::Shared<'a>
///     ) -> Result<Self::MainThread<'a>, PluginError> {
///         Ok(())
///     }
/// }
///
/// clack_export_entry!(SinglePluginEntry::<MyPlugin>);
/// ```
pub struct SinglePluginEntry<P: SimplePlugin> {
    plugin_factory: PluginFactoryWrapper<SinglePluginFactory<P>>,
}

impl<P: SimplePlugin> Entry for SinglePluginEntry<P> {
    fn new(_plugin_path: &CStr) -> Result<Self, EntryLoadError> {
        Ok(Self {
            plugin_factory: PluginFactoryWrapper::new(SinglePluginFactory {
                descriptor: P::get_descriptor(),
                _plugin: PhantomData,
            }),
        })
    }

    #[inline]
    fn declare_factories<'a>(&'a self, builder: &mut EntryFactories<'a>) {
        builder.register_factory(&self.plugin_factory);
    }
}

struct SinglePluginFactory<P> {
    descriptor: PluginDescriptor,
    _plugin: PhantomData<fn() -> P>,
}

impl<P: SimplePlugin> PluginFactory for SinglePluginFactory<P> {
    #[inline]
    fn plugin_count(&self) -> u32 {
        1
    }

    #[inline]
    fn plugin_descriptor(&self, index: u32) -> Option<&PluginDescriptor> {
        match index {
            0 => Some(&self.descriptor),
            _ => None,
        }
    }

    #[inline]
    fn instantiate_plugin<'a>(
        &'a self,
        host_info: HostInfo<'a>,
        plugin_id: &CStr,
    ) -> Option<PluginInstance<'a>> {
        if plugin_id == self.descriptor.id() {
            Some(PluginInstance::new::<P>(
                host_info,
                &self.descriptor,
                P::new_shared,
                P::new_main_thread,
            ))
        } else {
            None
        }
    }
}

/// A trait used by [`SinglePluginEntry`] that provides simplified, generic methods for plugin
/// instance creation.
///
/// See the [`SinglePluginEntry`] documentation for more information and examples.
pub trait SimplePlugin: Plugin {
    /// Returns a new Plugin Descriptor, which contains metadata about the plugin, such as its name,
    /// stable identifier, and more.
    ///
    /// See the [`PluginDescriptor`] type's documentation for more information.
    fn get_descriptor() -> PluginDescriptor;

    /// Creates a new instance of this shared data.
    ///
    /// This struct receives a thread-safe host handle that can be stored for the lifetime of the plugin.
    ///
    /// # Errors
    /// This operation may fail for any reason, in which case `Err` is returned and the plugin is
    /// not instantiated.
    fn new_shared(host: HostHandle) -> Result<Self::Shared<'_>, PluginError>;

    /// Creates a new instance of the plugin's main thread.
    ///
    /// This struct receives an exclusive host handle that can be stored for the lifetime of the plugin.
    ///
    /// # Errors
    /// This operation may fail for any reason, in which case `Err` is returned and the plugin is
    /// not instantiated.
    fn new_main_thread<'a>(
        host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError>;
}
