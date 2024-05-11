use crate::extensions::wrapper::descriptor::RawHostDescriptor;
use crate::extensions::wrapper::HostWrapper;
use crate::prelude::*;
use clap_sys::plugin::clap_plugin;
use std::ffi::CStr;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub(crate) struct PluginInstanceInner<H: HostHandlers> {
    host_wrapper: Pin<Arc<HostWrapper<H>>>,
    host_descriptor: Pin<Box<RawHostDescriptor>>,
    plugin_ptr: Option<NonNull<clap_plugin>>,

    is_started: AtomicBool,

    _plugin_bundle: PluginBundle, // SAFETY: Keep the DLL/.SO alive while plugin is instantiated
}

impl<H: HostHandlers> PluginInstanceInner<H> {
    pub(crate) fn instantiate<FH, FS>(
        shared: FS,
        main_thread: FH,
        plugin_bundle: &PluginBundle,
        plugin_id: &CStr,
        host_info: HostInfo,
    ) -> Result<Arc<Self>, HostError>
    where
        FS: for<'s> FnOnce(&'s ()) -> <H as HostHandlers>::Shared<'s>,
        FH: for<'s> FnOnce(
            &'s <H as HostHandlers>::Shared<'s>,
        ) -> <H as HostHandlers>::MainThread<'s>,
    {
        let plugin_factory = plugin_bundle
            .get_plugin_factory()
            .ok_or(HostError::MissingPluginFactory)?;

        let host_wrapper = HostWrapper::new(shared, main_thread);
        let host_descriptor = Box::pin(RawHostDescriptor::new::<H>(host_info));

        let mut instance = Arc::new(Self {
            host_wrapper,
            host_descriptor,
            plugin_ptr: None,
            _plugin_bundle: plugin_bundle.clone(),
            is_started: AtomicBool::new(false),
        });

        {
            let instance = Arc::get_mut(&mut instance).unwrap();
            instance.host_descriptor.set_wrapper(&instance.host_wrapper);

            let raw_descriptor = instance.host_descriptor.raw();

            // SAFETY: the host pointer comes from a valid allocation that is pinned for the
            // lifetime of the instance
            let plugin_instance_ptr =
                unsafe { plugin_factory.create_plugin(plugin_id, raw_descriptor)? };

            // SAFETY: The pointer comes from the plugin factory
            unsafe {
                instance.host_wrapper.created(plugin_instance_ptr);
            }

            // Now instantiate the plugin
            // SAFETY: The CLAP spec requires those function pointers to be valid to call
            unsafe {
                if let Some(init) = plugin_instance_ptr.as_ref().init {
                    if !init(plugin_instance_ptr.as_ptr()) {
                        instance.host_wrapper.start_instance_destroy();
                        if let Some(destroy) = plugin_instance_ptr.as_ref().destroy {
                            destroy(plugin_instance_ptr.as_ptr());
                        }

                        return Err(HostError::InstantiationFailed);
                    }
                }
            }

            // SAFETY: The pointer comes from the plugin factory
            unsafe { instance.host_wrapper.instantiated() };
            instance.plugin_ptr = Some(plugin_instance_ptr);
        }

        Ok(instance)
    }

    #[inline]
    pub fn wrapper(&self) -> &HostWrapper<H> {
        &self.host_wrapper
    }

    #[inline]
    pub fn raw_instance(&self) -> &clap_plugin {
        // SAFETY: This can only be None in the middle of instantiate()
        unsafe { self.plugin_ptr.unwrap_unchecked().as_ref() }
    }

    #[inline]
    pub fn plugin_shared(&self) -> PluginSharedHandle {
        // SAFETY: the raw instance is guaranteed to be valid
        unsafe { PluginSharedHandle::new(self.raw_instance().into()) }
    }

    pub fn activate<FA>(
        &mut self,
        audio_processor: FA,
        configuration: PluginAudioConfiguration,
    ) -> Result<(), HostError>
    where
        FA: for<'a> FnOnce(
            &'a <H as HostHandlers>::Shared<'a>,
            &mut <H as HostHandlers>::MainThread<'a>,
        ) -> <H as HostHandlers>::AudioProcessor<'a>,
    {
        let activate = self
            .raw_instance()
            .activate
            .ok_or(HostError::NullActivateFunction)?;

        // SAFETY: this method being &mut guarantees nothing can call any other main-thread method
        unsafe {
            self.host_wrapper.setup_audio_processor(audio_processor)?;
        }

        // SAFETY: this type ensures the function pointer is valid
        let success = unsafe {
            activate(
                self.raw_instance(),
                configuration.sample_rate,
                configuration.min_frames_count,
                configuration.max_frames_count,
            )
        };

        if !success {
            // SAFETY: this method being &mut guarantees nothing can call any other main-thread method
            let _ = unsafe { self.host_wrapper.teardown_audio_processor(|_, _| ()) };
            return Err(HostError::ActivationFailed);
        }

        Ok(())
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.wrapper().is_active()
    }

    #[inline]
    pub fn deactivate_with<T>(
        &mut self,
        drop: impl for<'s> FnOnce(
            <H as HostHandlers>::AudioProcessor<'s>,
            &mut <H as HostHandlers>::MainThread<'s>,
        ) -> T,
    ) -> Result<T, HostError> {
        if !self.is_active() {
            return Err(HostError::DeactivatedPlugin);
        }

        if self.is_started.load(Ordering::Acquire) {
            // SAFETY: this method being &mut guarantees nothing can call any other main-thread method
            unsafe { self.stop_processing() }
        }

        if let Some(deactivate) = self.raw_instance().deactivate {
            // SAFETY: this type ensures the function pointer is valid.
            // We just checked the instance is in an active state.
            unsafe { deactivate(self.raw_instance()) };
        }

        // SAFETY: this method being &mut guarantees nothing can call any other main-thread method
        unsafe { self.host_wrapper.teardown_audio_processor(drop) }
    }

    /// # Safety
    /// User must ensure the instance is not in a processing state, and that this is only called
    /// on the audio thread.
    #[inline]
    pub unsafe fn start_processing(&self) -> Result<(), HostError> {
        if let Some(start_processing) = self.raw_instance().start_processing {
            if start_processing(self.raw_instance()) {
                self.is_started.store(true, Ordering::Release);
                return Ok(());
            }

            Err(HostError::StartProcessingFailed)
        } else {
            Ok(())
        }
    }

    /// # Safety
    /// User must ensure that this is only called on the audio thread.
    #[inline]
    pub unsafe fn reset(&self) {
        if let Some(reset) = self.raw_instance().reset {
            reset(self.raw_instance())
        }
    }

    /// # Safety
    /// User must ensure the instance is in a processing state, and that this is only called
    /// on the audio thread.
    #[inline]
    pub unsafe fn stop_processing(&self) {
        if let Some(stop_processing) = self.raw_instance().stop_processing {
            stop_processing(self.raw_instance());
            self.is_started.store(false, Ordering::Release);
        }
    }

    /// # Safety
    /// User must ensure this is only called on the main thread.
    #[inline]
    pub unsafe fn on_main_thread(&self) {
        if let Some(on_main_thread) = self.raw_instance().on_main_thread {
            on_main_thread(self.raw_instance())
        }
    }
}

impl<H: HostHandlers> Drop for PluginInstanceInner<H> {
    #[inline]
    fn drop(&mut self) {
        // Happens only if instantiate didn't complete
        let Some(plugin_ptr) = self.plugin_ptr else {
            return;
        };

        // Check if instance hasn't been properly deactivated
        if self.is_active() {
            let _ = self.deactivate_with(|_, _| ());
        }

        self.host_wrapper.start_instance_destroy();
        // SAFETY: we are in the drop impl, so this can only be called once and without any
        // other concurrent calls.
        // This type also ensures the function pointer type is valid.
        unsafe {
            if let Some(destroy) = plugin_ptr.as_ref().destroy {
                destroy(plugin_ptr.as_ptr())
            }
        }
    }
}

// SAFETY: The only non-thread-safe methods on this type are unsafe
unsafe impl<H: HostHandlers> Send for PluginInstanceInner<H> {}
// SAFETY: The only non-thread-safe methods on this type are unsafe
unsafe impl<H: HostHandlers> Sync for PluginInstanceInner<H> {}
