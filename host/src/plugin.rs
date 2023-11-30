use crate::prelude::*;
use clap_sys::plugin::clap_plugin;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::sync::Arc;

mod handle;
pub(crate) mod instance;

use crate::util::{WeakReader, WriterLock};
pub use handle::*;
use instance::*;

/// A plugin instance.
pub struct PluginInstance<H: Host> {
    inner: WriterLock<Arc<PluginInstanceInner<H>>>,
    _no_send: PhantomData<*const ()>,
}

impl<H: Host> PluginInstance<H> {
    pub fn new<FS, FH>(
        shared: FS,
        main_thread: FH,
        bundle: &PluginBundle,
        plugin_id: &CStr,
        host: &HostInfo,
    ) -> Result<Self, HostError>
    where
        FS: for<'b> FnOnce(&'b ()) -> <H as Host>::Shared<'b>,
        FH: for<'b> FnOnce(&'b <H as Host>::Shared<'b>) -> <H as Host>::MainThread<'b>,
    {
        let inner = PluginInstanceInner::<H>::instantiate(
            shared,
            main_thread,
            bundle,
            plugin_id,
            host.clone(),
        )?;

        Ok(Self {
            inner: WriterLock::new(inner),
            _no_send: PhantomData,
        })
    }

    pub fn activate<FA>(
        &mut self,
        audio_processor: FA,
        configuration: PluginAudioConfiguration,
    ) -> Result<StoppedPluginAudioProcessor<H>, HostError>
    where
        FA: for<'a> FnOnce(
            PluginAudioProcessorHandle<'a>,
            &'a <H as Host>::Shared<'a>,
            &mut <H as Host>::MainThread<'a>,
        ) -> <H as Host>::AudioProcessor<'a>,
    {
        self.inner.use_mut(|inner| {
            let wrapper = Arc::get_mut(inner).ok_or(HostError::AlreadyActivatedPlugin)?;

            wrapper.activate(audio_processor, configuration)?;

            Ok(StoppedPluginAudioProcessor::new(inner.clone()))
        })
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
        D: for<'s> FnOnce(<H as Host>::AudioProcessor<'s>, &mut <H as Host>::MainThread<'s>) -> T,
    {
        let inner = self.inner.get();
        if !Arc::ptr_eq(inner, &processor.inner) {
            panic!("Given plugin audio processor does not match the instance being deactivated")
        }

        drop(processor);

        // PANIC: we dropped the only processor produced, and checked if it matched
        self.try_deactivate_with(drop_with).unwrap()
    }

    pub fn try_deactivate_with<T, D>(&mut self, drop_with: D) -> Result<T, HostError>
    where
        D: for<'s> FnOnce(<H as Host>::AudioProcessor<'s>, &mut <H as Host>::MainThread<'s>) -> T,
    {
        self.inner.use_mut(|inner| {
            let wrapper = Arc::get_mut(inner).ok_or(HostError::StillActivatedPlugin)?;

            wrapper.deactivate_with(drop_with)
        })
    }

    #[inline]
    pub fn call_on_main_thread_callback(&mut self) {
        // SAFETY: this is done on the main thread, and the &mut reference guarantees no aliasing
        unsafe { self.inner.get().on_main_thread() }
    }

    #[inline]
    pub fn raw_instance(&self) -> &clap_plugin {
        self.inner.get().raw_instance()
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        Arc::strong_count(self.inner.get()) > 1
    }

    #[inline]
    pub fn shared_host_data(&self) -> &<H as Host>::Shared<'_> {
        self.inner.get().wrapper().shared()
    }

    #[inline]
    pub fn main_thread_host_data(&self) -> &<H as Host>::MainThread<'_> {
        // SAFETY: we take &self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.inner.get().wrapper().main_thread().as_ref() }
    }

    #[inline]
    pub fn main_thread_host_data_mut(&mut self) -> &mut <H as Host>::MainThread<'_> {
        // SAFETY: we take &mut self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.inner.get().wrapper().main_thread().as_mut() }
    }

    #[inline]
    pub fn shared_plugin_data(&self) -> PluginSharedHandle {
        self.inner.get().plugin_shared()
    }

    #[inline]
    pub fn main_thread_plugin_data(&mut self) -> PluginMainThreadHandle {
        // SAFETY: we are on the main thread, and the &mut reference guarantees uniqueness.
        unsafe { self.inner.get().plugin_main_thread() }
    }
}

pub struct PluginInstanceHandle<H: Host> {
    inner: WeakReader<Arc<PluginInstanceInner<H>>>,
}

impl<H: Host> PluginInstanceHandle<H> {
    #[inline]
    pub fn use_shared_host_data<T>(
        &self,
        lambda: impl FnOnce(&H::Shared<'_>) -> T,
    ) -> Result<T, HostError> {
        self.inner
            .use_with(|inner| lambda(inner.wrapper().shared()))
            .ok_or(HostError::PluginDestroyed)
    }

    #[inline]
    pub fn use_shared_plugin_data<T>(
        &self,
        lambda: impl FnOnce(PluginSharedHandle) -> T,
    ) -> Result<T, HostError> {
        self.inner
            .use_with(|inner| lambda(inner.plugin_shared()))
            .ok_or(HostError::PluginDestroyed)
    }
}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_any!(PluginInstance<()>: Send, Sync);
}
