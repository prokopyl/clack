use crate::entry::prelude::*;
use crate::prelude::*;
use std::ffi::CStr;
use std::marker::PhantomData;

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
    _plugin: PhantomData<P>,
}

unsafe impl<P> Send for SinglePluginFactory<P> {}
unsafe impl<P> Sync for SinglePluginFactory<P> {}

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
    fn create_plugin<'a>(
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
