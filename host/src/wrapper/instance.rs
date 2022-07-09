use crate::bundle::PluginBundle;
use crate::factory::PluginFactory;
use crate::host::{Host, HostInfo};
use crate::instance::PluginAudioConfiguration;
use crate::plugin::PluginAudioProcessorHandle;
use crate::wrapper::descriptor::{RawHostDescriptor, RawHostDescriptorRef};
use crate::wrapper::{HostError, HostWrapper};
use clap_sys::plugin::clap_plugin;
use selfie::Selfie;
use stable_deref_trait::StableDeref;
use std::marker::PhantomData;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;

pub struct RawPluginInstanceRef(PhantomData<clap_plugin>);

impl Default for RawPluginInstanceRef {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl Deref for RawPluginInstanceRef {
    type Target = ();

    #[inline]
    fn deref(&self) -> &Self::Target {
        &()
    }
}

// SAFETY: this derefs to nothing
unsafe impl StableDeref for RawPluginInstanceRef {}

pub struct PluginInstanceInner<H: for<'a> Host<'a>> {
    host_descriptor: Selfie<'static, Box<HostWrapper<H>>, Pin<Box<RawHostDescriptorRef>>>,
    instance: *mut clap_plugin,
    _plugin_bundle: PluginBundle, // Keep the DLL/.SO alive while plugin is instantiated
}

impl<H: for<'a> Host<'a>> PluginInstanceInner<H> {
    pub(crate) fn instantiate<FH, FS>(
        shared: FS,
        main_thread: FH,
        entry: &PluginBundle,
        plugin_id: &[u8],
        host_info: HostInfo,
    ) -> Result<Arc<Self>, HostError>
    where
        FS: for<'s> FnOnce(&'s ()) -> <H as Host<'s>>::Shared,
        FH: for<'s> FnOnce(&'s <H as Host<'s>>::Shared) -> <H as Host<'s>>::MainThread,
    {
        let host_wrapper = Box::pin(HostWrapper::new(shared, main_thread));
        let host_descriptor = Selfie::new(host_wrapper, |w| {
            Box::pin(RawHostDescriptor::new(host_info, w))
        });

        let raw_descriptor =
            host_descriptor.with_referential(|d: &Pin<Box<RawHostDescriptor>>| d.raw() as *const _);

        let instance = unsafe {
            entry
                .get_factory::<PluginFactory>()
                .ok_or(HostError::MissingPluginFactory)?
                .instantiate(plugin_id, &*raw_descriptor)?
                .as_ptr()
        };

        host_descriptor.owned().instantiated(instance);

        Ok(Arc::new(Self {
            host_descriptor,
            instance,
            _plugin_bundle: entry.clone(),
        }))
    }

    #[inline]
    pub fn wrapper(&self) -> &HostWrapper<H> {
        self.host_descriptor.owned()
    }

    #[inline]
    pub fn raw_instance(&self) -> &clap_plugin {
        unsafe { &*self.instance }
    }

    #[inline]
    pub fn raw_instance_mut(&mut self) -> &mut clap_plugin {
        unsafe { &mut *self.instance }
    }

    pub fn activate<FA>(
        &mut self,
        audio_processor: FA,
        configuration: PluginAudioConfiguration,
    ) -> Result<(), HostError>
    where
        FA: for<'a> FnOnce(
            PluginAudioProcessorHandle<'a>,
            &'a <H as Host<'a>>::Shared,
            &mut <H as Host<'a>>::MainThread,
        ) -> <H as Host<'a>>::AudioProcessor,
    {
        if self.wrapper().is_active() {
            return Err(HostError::AlreadyActivatedPlugin);
        }

        unsafe {
            self.wrapper()
                .activate(audio_processor, self.raw_instance())
        };

        let success = unsafe {
            ((*self.instance).activate)(
                self.instance,
                configuration.sample_rate,
                *configuration.frames_count_range.start(),
                *configuration.frames_count_range.end(),
            )
        };

        if !success {
            unsafe { self.wrapper().deactivate(|_, _| ()) };
            return Err(HostError::ActivationFailed);
        }

        Ok(())
    }

    #[inline]
    pub fn deactivate_with<T>(
        &mut self,
        drop: impl for<'s> FnOnce(
            <H as Host<'s>>::AudioProcessor,
            &mut <H as Host<'s>>::MainThread,
        ) -> T,
    ) -> Result<T, HostError> {
        if !self.wrapper().is_active() {
            return Err(HostError::DeactivatedPlugin);
        }

        unsafe { ((*self.instance).deactivate)(self.instance) };

        Ok(unsafe { self.wrapper().deactivate(drop) })
    }

    #[inline]
    pub unsafe fn start_processing(&self) -> Result<(), HostError> {
        if ((*self.instance).start_processing)(self.instance) {
            return Ok(());
        }

        Err(HostError::StartProcessingFailed)
    }

    #[inline]
    pub unsafe fn stop_processing(&self) {
        ((*self.instance).stop_processing)(self.instance)
    }

    #[inline]
    pub unsafe fn on_main_thread(&self) {
        ((*self.instance).on_main_thread)(self.instance)
    }
}

impl<H: for<'h> Host<'h>> Drop for PluginInstanceInner<H> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ((*self.raw_instance()).destroy)(self.raw_instance_mut() as *mut _) }
    }
}

// SAFETY: The only non-thread-safe methods on this type are unsafe
unsafe impl<H: for<'h> Host<'h>> Send for PluginInstanceInner<H> {}
unsafe impl<H: for<'h> Host<'h>> Sync for PluginInstanceInner<H> {}
