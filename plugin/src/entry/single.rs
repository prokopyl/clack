use crate::entry::prelude::*;
use crate::prelude::*;
use std::ffi::CStr;
use std::marker::PhantomData;

/// An [`Entry`] that only exposes a single plugin type to the host.
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
/// use clack_plugin::prelude::*;
///
/// pub struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     /* ... */
/// #   type AudioProcessor<'a> = (); type Shared<'a> = (); type MainThread<'a> = ();
/// #   fn get_descriptor() -> Box<dyn PluginDescriptor> {
/// #       unreachable!()
/// #   }
/// }
///
/// clack_export_entry!(SinglePluginEntry::<MyPlugin>);
/// ```
pub struct SinglePluginEntry<P> {
    plugin_factory: PluginFactoryWrapper<SinglePluginFactory<P>>,
}

impl<P: Plugin> Entry for SinglePluginEntry<P> {
    fn new(_plugin_path: &CStr) -> Result<Self, EntryLoadError> {
        Ok(Self {
            plugin_factory: PluginFactoryWrapper::new(SinglePluginFactory {
                descriptor: PluginDescriptorWrapper::new(P::get_descriptor()),
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
    descriptor: PluginDescriptorWrapper,
    _plugin: PhantomData<fn(P) -> P>,
}

impl<P: Plugin> PluginFactory for SinglePluginFactory<P> {
    #[inline]
    fn plugin_count(&self) -> u32 {
        1
    }

    #[inline]
    fn plugin_descriptor(&self, index: u32) -> Option<&PluginDescriptorWrapper> {
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
        if plugin_id == self.descriptor.descriptor().id() {
            Some(PluginInstance::new::<P>(host_info, &self.descriptor))
        } else {
            None
        }
    }
}
