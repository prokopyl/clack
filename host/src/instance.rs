use crate::bundle::PluginBundle;
use crate::host::{Host, HostInfo};
use clap_sys::plugin::clap_plugin;
use std::ffi::CStr;
use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::extensions::wrapper::instance::PluginInstanceInner;
use crate::host::HostError;
use crate::instance::handle::{
    PluginAudioProcessorHandle, PluginMainThreadHandle, PluginSharedHandle,
};
use crate::instance::processor::StoppedPluginAudioProcessor;

pub mod handle;
pub mod processor;

pub struct PluginAudioConfiguration {
    pub sample_rate: f64,
    pub frames_count_range: RangeInclusive<u32>,
}

pub struct PluginInstance<H: for<'a> Host<'a>> {
    inner: Arc<PluginInstanceInner<H>>,
}

impl<H: for<'b> Host<'b>> PluginInstance<H> {
    pub fn new<FS, FH>(
        shared: FS,
        main_thread: FH,
        bundle: &PluginBundle,
        plugin_id: &CStr,
        host: &HostInfo,
    ) -> Result<Self, HostError>
    where
        FS: for<'b> FnOnce(&'b ()) -> <H as Host<'b>>::Shared,
        FH: for<'b> FnOnce(&'b <H as Host<'b>>::Shared) -> <H as Host<'b>>::MainThread,
    {
        let inner = PluginInstanceInner::<H>::instantiate(
            shared,
            main_thread,
            bundle,
            plugin_id,
            host.clone(),
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
            PluginAudioProcessorHandle<'a>,
            &'a <H as Host<'a>>::Shared,
            &mut <H as Host<'a>>::MainThread,
        ) -> <H as Host<'a>>::AudioProcessor,
    {
        let wrapper = Arc::get_mut(&mut self.inner).ok_or(HostError::AlreadyActivatedPlugin)?;

        wrapper.activate(audio_processor, configuration)?;

        Ok(StoppedPluginAudioProcessor::new(self.inner.clone()))
    }

    #[inline]
    pub fn deactivate(&mut self, processor: StoppedPluginAudioProcessor<H>) {
        self.deactivate_with(processor, |_, _| ())
    }

    #[inline]
    pub fn try_deactivate(&mut self) -> Result<(), HostError> {
        self.try_deactivate_with(|_, _| ())
    }

    pub fn deactivate_with<T, D>(
        &mut self,
        processor: StoppedPluginAudioProcessor<H>,
        drop_with: D,
    ) -> T
    where
        D: for<'s> FnOnce(<H as Host<'s>>::AudioProcessor, &mut <H as Host<'s>>::MainThread) -> T,
    {
        if !Arc::ptr_eq(&self.inner, &processor.inner) {
            panic!("Given plugin audio processor does not match the instance being deactivated")
        }

        drop(processor);

        // PANIC: we dropped the only processor produced, and checked if it matched
        self.try_deactivate_with(drop_with).unwrap()
    }

    pub fn try_deactivate_with<T, D>(&mut self, drop_with: D) -> Result<T, HostError>
    where
        D: for<'s> FnOnce(<H as Host<'s>>::AudioProcessor, &mut <H as Host<'s>>::MainThread) -> T,
    {
        let wrapper = Arc::get_mut(&mut self.inner).ok_or(HostError::StillActivatedPlugin)?;

        wrapper.deactivate_with(drop_with)
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
    pub fn shared_host_data(&self) -> &<H as Host>::Shared {
        self.inner.wrapper().shared()
    }

    #[inline]
    pub fn main_thread_host_data(&self) -> &<H as Host>::MainThread {
        // SAFETY: we take &self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.inner.wrapper().main_thread().as_ref() }
    }

    #[inline]
    pub fn main_thread_host_data_mut(&mut self) -> &mut <H as Host>::MainThread {
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
