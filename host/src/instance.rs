use crate::bundle::PluginBundle;
use crate::host::{PluginHost, PluginHoster};
use clap_sys::plugin::clap_plugin;
use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::instance::processor::StoppedPluginAudioProcessor;
use crate::plugin::{PluginMainThreadHandle, PluginSharedHandle};
use crate::wrapper::instance::PluginInstanceInner;
use crate::wrapper::HostError;

pub struct PluginAudioConfiguration {
    pub sample_rate: f64,
    pub frames_count_range: RangeInclusive<u32>,
}

pub struct PluginInstance<H: for<'a> PluginHoster<'a>> {
    inner: Arc<PluginInstanceInner<H>>,
}

pub mod processor;

impl<H: for<'b> PluginHoster<'b>> PluginInstance<H> {
    pub fn new<FS, FH>(
        shared: FS,
        main_thread: FH,
        bundle: &PluginBundle,
        plugin_id: &[u8],
        host: &PluginHost,
    ) -> Result<Self, HostError>
    where
        FS: for<'b> FnOnce(&'b ()) -> <H as PluginHoster<'b>>::Shared,
        FH: for<'b> FnOnce(
            &'b <H as PluginHoster<'b>>::Shared,
        ) -> <H as PluginHoster<'b>>::MainThread,
    {
        let inner = PluginInstanceInner::<H>::instantiate(
            shared,
            main_thread,
            bundle,
            plugin_id,
            host.shared().clone(),
        )?;

        Ok(Self { inner })
    }

    pub fn activate<FA>(
        &mut self,
        audio_processor: FA,
        configuration: PluginAudioConfiguration,
    ) -> Result<StoppedPluginAudioProcessor<H>, HostError>
    where
        FA: for<'a> FnOnce(
            &'a <H as PluginHoster<'a>>::Shared,
            &mut <H as PluginHoster<'a>>::MainThread,
        ) -> <H as PluginHoster<'a>>::AudioProcessor,
    {
        let wrapper = Arc::get_mut(&mut self.inner).ok_or(HostError::AlreadyActivatedPlugin)?;

        wrapper.activate(audio_processor, configuration)?;

        Ok(StoppedPluginAudioProcessor::new(self.inner.clone()))
    }

    pub fn deactivate(&mut self, processor: StoppedPluginAudioProcessor<H>) {
        // SAFETY: we never clone the arcs, only compare them
        if !Arc::ptr_eq(&self.inner, &processor.inner) {
            panic!("Given plugin audio processor does not match the instance being deactivated")
        }

        drop(processor);

        // PANIC: we dropped the only processor produced, and checked if it matched
        let wrapper = Arc::get_mut(&mut self.inner)
            .ok_or(HostError::AlreadyActivatedPlugin)
            .unwrap();

        // PANIC: we dropped the only processor produced, and checked if it matched
        wrapper.deactivate().unwrap();
    }

    #[inline]
    pub fn call_on_main_thread_callback(&mut self) {
        // SAFETY: this is done on the main thread, and the &mut reference guarantees no aliasing
        unsafe { self.inner.on_main_thread() }
    }

    #[inline]
    pub fn raw_instance(&self) -> &clap_plugin {
        self.inner.raw_instance()
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        Arc::strong_count(&self.inner) > 1
    }

    #[inline]
    pub fn shared_host_data(&self) -> &<H as PluginHoster>::Shared {
        self.inner.wrapper().shared()
    }

    #[inline]
    pub fn main_thread_host_data(&self) -> &<H as PluginHoster>::MainThread {
        // SAFETY: we take &self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.inner.wrapper().main_thread().as_ref() }
    }

    #[inline]
    pub fn main_thread_host_data_mut(&mut self) -> &mut <H as PluginHoster>::MainThread {
        // SAFETY: we take &mut self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.inner.wrapper().main_thread().as_mut() }
    }

    #[inline]
    pub fn shared_plugin_data(&self) -> PluginSharedHandle {
        PluginSharedHandle::new((self.inner.raw_instance() as *const _) as *mut _)
    }

    #[inline]
    pub fn main_thread_plugin_data(&self) -> PluginMainThreadHandle {
        PluginMainThreadHandle::new((self.inner.raw_instance() as *const _) as *mut _)
    }
}

impl<H: for<'h> PluginHoster<'h>> Drop for PluginInstance<H> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ((*self.inner.raw_instance()).destroy)(self.inner.raw_instance()) }
    }
}
