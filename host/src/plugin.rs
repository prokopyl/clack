use crate::prelude::*;
use clap_sys::plugin::clap_plugin;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::sync::Arc;

mod error;
mod handle;
pub(crate) mod instance;

pub use error::PluginInstanceError;
pub use handle::*;
use instance::*;

pub use clack_common::plugin::*;

/// A plugin instance.
pub struct PluginInstance<H: HostHandlers> {
    pub(crate) inner: ManuallyDrop<Arc<PluginInstanceInner<H>>>,
    _no_send: PhantomData<*const ()>,
}

impl<H: HostHandlers> PluginInstance<H> {
    pub fn new<FS, FH>(
        shared: FS,
        main_thread: FH,
        bundle: &PluginBundle,
        plugin_id: &CStr,
        host: &HostInfo,
    ) -> Result<Self, PluginInstanceError>
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
            inner: ManuallyDrop::new(inner),
            _no_send: PhantomData,
        })
    }

    pub fn activate<FA>(
        &mut self,
        audio_processor: FA,
        configuration: PluginAudioConfiguration,
    ) -> Result<StoppedPluginAudioProcessor<H>, PluginInstanceError>
    where
        FA: for<'a> FnOnce(
            &'a <H as HostHandlers>::Shared<'a>,
            &mut <H as HostHandlers>::MainThread<'a>,
        ) -> <H as HostHandlers>::AudioProcessor<'a>,
    {
        let wrapper =
            Arc::get_mut(&mut self.inner).ok_or(PluginInstanceError::AlreadyActivatedPlugin)?;
        wrapper.activate(audio_processor, configuration)?;

        Ok(StoppedPluginAudioProcessor::new(Arc::clone(&self.inner)))
    }

    #[inline]
    pub fn deactivate(&mut self, processor: StoppedPluginAudioProcessor<H>) {
        self.deactivate_with(processor, |_, _| ())
    }

    #[inline]
    pub fn try_deactivate(&mut self) -> Result<(), PluginInstanceError> {
        self.try_deactivate_with(|_, _| ())
    }

    pub fn deactivate_with<T, D>(
        &mut self,
        processor: StoppedPluginAudioProcessor<H>,
        drop_with: D,
    ) -> T
    where
        D: for<'s> FnOnce(
            <H as HostHandlers>::AudioProcessor<'s>,
            &mut <H as HostHandlers>::MainThread<'s>,
        ) -> T,
    {
        if !Arc::ptr_eq(&self.inner, &processor.inner) {
            panic!("Given plugin audio processor does not match the instance being deactivated")
        }

        drop(processor);

        // PANIC: we dropped the only processor produced, and checked if it matched
        self.try_deactivate_with(drop_with).unwrap()
    }

    pub fn try_deactivate_with<T, D>(&mut self, drop_with: D) -> Result<T, PluginInstanceError>
    where
        D: for<'s> FnOnce(
            <H as HostHandlers>::AudioProcessor<'s>,
            &mut <H as HostHandlers>::MainThread<'s>,
        ) -> T,
    {
        let wrapper =
            Arc::get_mut(&mut self.inner).ok_or(PluginInstanceError::StillActivatedPlugin)?;

        wrapper.deactivate_with(drop_with)
    }

    // FIXME: this should be on the handle?
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
        self.inner.is_active()
    }

    #[inline]
    pub fn access_shared_handler<'s, R>(
        &'s self,
        access: impl for<'a> FnOnce(&'s <H as HostHandlers>::Shared<'a>) -> R,
    ) -> R {
        access(self.inner.wrapper().shared())
    }

    #[inline]
    pub fn access_handler<'s, R>(
        &'s self,
        access: impl for<'a> FnOnce(&'s <H as HostHandlers>::MainThread<'a>) -> R,
    ) -> R {
        // SAFETY: we take &self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { access(self.inner.wrapper().main_thread().as_ref()) }
    }

    #[inline]
    pub fn access_handler_mut<'s, R>(
        &'s mut self,
        access: impl for<'a> FnOnce(&'s mut <H as HostHandlers>::MainThread<'a>) -> R,
    ) -> R {
        // SAFETY: we take &mut self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { access(self.inner.wrapper().main_thread().as_mut()) }
    }

    #[inline]
    pub fn plugin_shared_handle(&self) -> PluginSharedHandle<'_> {
        self.inner.plugin_shared()
    }

    #[inline]
    pub fn plugin_handle(&mut self) -> PluginMainThreadHandle<'_> {
        // SAFETY: this type can only exist on the main thread.
        unsafe { PluginMainThreadHandle::new(self.inner.raw_instance().into()) }
    }
}

impl<H: HostHandlers> Drop for PluginInstance<H> {
    fn drop(&mut self) {
        // Only drop our Arc if we are the sole owner.
        // This leaks the plugin instance, but prevents accidentally transferring ownership to the
        // audio thread if the audio processor handle is still around somewhere.
        if Arc::get_mut(&mut self.inner).is_some() {
            // SAFETY: We can only call this once (as we're in Drop), and we never use the inner
            // value again afterward.
            unsafe { ManuallyDrop::drop(&mut self.inner) }
        };
    }
}

#[cfg(test)]
mod test {
    extern crate static_assertions as sa;
    use super::*;

    sa::assert_not_impl_any!(PluginInstance<()>: Send, Sync);
}
