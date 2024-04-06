use crate::prelude::*;
use clap_sys::plugin::clap_plugin;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::sync::Arc;

mod handle;
pub(crate) mod instance;

pub use handle::*;
use instance::*;

use crate::util::{WeakReader, WriterLock};
pub use clack_common::plugin::*;

/// A plugin instance.
pub struct PluginInstance<H: HostHandlers> {
    pub(crate) inner: WriterLock<PluginInstanceInner<H>>,
    _no_send: PhantomData<*const ()>,
}

impl<H: HostHandlers> PluginInstance<H> {
    pub fn new<FS, FH>(
        shared: FS,
        main_thread: FH,
        bundle: &PluginBundle,
        plugin_id: &CStr,
        host: &HostInfo,
    ) -> Result<Self, HostError>
    where
        FS: for<'b> FnOnce(&'b ()) -> <H as HostHandlers>::Shared<'b>,
        FH: for<'b> FnOnce(
            &'b <H as HostHandlers>::Shared<'b>,
        ) -> <H as HostHandlers>::MainThread<'b>,
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
            &'a <H as HostHandlers>::Shared<'a>,
            &mut <H as HostHandlers>::MainThread<'a>,
        ) -> <H as HostHandlers>::AudioProcessor<'a>,
    {
        self.inner.use_mut(|inner| {
            let wrapper = Arc::get_mut(inner).ok_or(HostError::AlreadyActivatedPlugin)?;

            wrapper.activate(audio_processor, configuration)
        })?;

        Ok(StoppedPluginAudioProcessor::new(Arc::clone(
            self.inner.get(),
        )))
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
        D: for<'s> FnOnce(
            <H as HostHandlers>::AudioProcessor<'_>,
            &mut <H as HostHandlers>::MainThread<'s>,
        ) -> T,
    {
        if !Arc::ptr_eq(self.inner.get(), &processor.inner) {
            panic!("Given plugin audio processor does not match the instance being deactivated")
        }

        drop(processor);

        // PANIC: we dropped the only processor produced, and checked if it matched
        self.try_deactivate_with(drop_with).unwrap()
    }

    pub fn try_deactivate_with<T, D>(&mut self, drop_with: D) -> Result<T, HostError>
    where
        D: for<'s> FnOnce(
            <H as HostHandlers>::AudioProcessor<'_>,
            &mut <H as HostHandlers>::MainThread<'s>,
        ) -> T,
    {
        self.inner.use_mut(|inner| {
            let wrapper = Arc::get_mut(inner).ok_or(HostError::StillActivatedPlugin)?;

            wrapper.deactivate_with(drop_with)
        })
    }

    // FIXME: this should be on the handle?
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
        self.inner.get().is_active()
    }

    #[inline]
    pub fn shared_handler(&self) -> &<H as HostHandlers>::Shared<'_> {
        self.inner.get().wrapper().shared()
    }

    #[inline]
    pub fn handler(&self) -> &<H as HostHandlers>::MainThread<'_> {
        // SAFETY: we take &self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.inner.get().wrapper().main_thread().as_ref() }
    }

    #[inline]
    pub fn handler_mut(&mut self) -> &mut <H as HostHandlers>::MainThread<'_> {
        // SAFETY: we take &mut self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.inner.get().wrapper().main_thread().as_mut() }
    }

    #[inline]
    pub fn plugin_shared_handle(&self) -> PluginSharedHandle {
        self.inner.get().plugin_shared()
    }

    #[inline]
    pub fn plugin_handle(&mut self) -> PluginMainThreadHandle {
        // SAFETY: this type can only exist on the main thread.
        unsafe { PluginMainThreadHandle::new(self.inner.get().raw_instance().into()) }
    }

    // TODO: bikeshed
    pub fn handle(&self) -> PluginInstanceHandle<H> {
        PluginInstanceHandle {
            inner: self.inner.make_reader(),
        }
    }
}

// TODO: bikeshed
pub struct PluginInstanceHandle<H: HostHandlers> {
    inner: WeakReader<PluginInstanceInner<H>>,
}

impl<H: HostHandlers> PluginInstanceHandle<H> {
    #[inline]
    pub fn use_handler<T>(&self, lambda: impl FnOnce(&H::Shared<'_>) -> T) -> Result<T, HostError> {
        self.inner
            .use_with(|inner| lambda(inner.wrapper().shared()))
            .ok_or(HostError::PluginDestroyed)
    }

    #[inline]
    pub fn use_plugin_handle<T>(
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
