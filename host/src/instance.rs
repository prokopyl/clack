use crate::bundle::PluginBundle;
use crate::host::{PluginHost, PluginHoster};
use clap_sys::plugin::clap_plugin;
use std::ops::RangeInclusive;
use std::pin::Pin;
use std::sync::Arc;

use crate::instance::processor::StoppedPluginAudioProcessor;
use crate::plugin::{PluginMainThreadHandle, PluginSharedHandle};
use crate::wrapper::{HostError, HostWrapper};

pub struct PluginAudioConfiguration {
    pub sample_rate: f64,
    pub frames_count_range: RangeInclusive<u32>,
}

pub struct PluginInstance<H: for<'a> PluginHoster<'a>> {
    wrapper: Pin<Arc<HostWrapper<H>>>,
}

pub mod processor;

#[inline]
unsafe fn pin_get_ptr<P>(pin: &Pin<P>) -> &P {
    // SAFETY: Pin is repr(transparent). The caller is responsible to ensure the pointer isn't cloned
    // outside of a Pin.
    &*(pin as *const _ as *const P)
}

#[inline]
unsafe fn pin_get_ptr_unchecked_mut<P>(pin: &mut Pin<P>) -> &mut P {
    // SAFETY: Pin is repr(transparent). The caller is responsible to ensure the data will never be
    // moved, similar to Pin::get_unchecked_mut
    &mut *(pin as *mut _ as *mut P)
}

#[inline]
pub(crate) fn arc_get_pin_mut<T>(pin: &mut Pin<Arc<T>>) -> Option<Pin<&mut T>> {
    // SAFETY: Arc::get_mut does not move anything
    let arc = unsafe { pin_get_ptr_unchecked_mut(pin) };
    let inner = Arc::get_mut(arc)?;

    // SAFETY: By using get_mut we guaranteed this is the only reference to it.
    // The &mut Pin<Arc<T>> argument guarantees this data was pinned, and the temporary Arc reference
    // is never exposed.
    unsafe { Some(Pin::new_unchecked(inner)) }
}

impl<H: for<'b> PluginHoster<'b>> PluginInstance<H> {
    pub fn new<FS, FH>(
        shared: FS,
        hoster: FH,
        bundle: &PluginBundle,
        plugin_id: &[u8],
        host: &PluginHost,
    ) -> Result<Self, HostError>
    where
        FS: for<'b> FnOnce(&'b ()) -> <H as PluginHoster<'b>>::Shared,
        FH: for<'b> FnOnce(&'b <H as PluginHoster<'b>>::Shared) -> H,
    {
        let wrapper = HostWrapper::new(hoster, shared, bundle, plugin_id, host.shared().clone())?;

        Ok(Self { wrapper })
    }

    pub fn activate<FA>(
        &mut self,
        audio_processor: FA,
        configuration: PluginAudioConfiguration,
    ) -> Result<StoppedPluginAudioProcessor<H>, HostError>
    where
        FA: for<'a> FnOnce(
            &'a <H as PluginHoster<'a>>::Shared,
            &mut H,
        ) -> <H as PluginHoster<'a>>::AudioProcessor,
    {
        let wrapper =
            arc_get_pin_mut(&mut self.wrapper).ok_or(HostError::AlreadyActivatedPlugin)?;

        wrapper.activate(audio_processor, configuration)?;

        Ok(StoppedPluginAudioProcessor::new(self.wrapper.clone()))
    }

    pub fn deactivate(&mut self, processor: StoppedPluginAudioProcessor<H>) {
        // SAFETY: we never clone the arcs, only compare them
        if unsafe { !Arc::ptr_eq(pin_get_ptr(&self.wrapper), pin_get_ptr(&processor.wrapper)) } {
            panic!("Given plugin audio processor does not match the instance being deactivated")
        }

        drop(processor);

        // PANIC: we dropped the only processor produced, and checked if it matched
        let wrapper = arc_get_pin_mut(&mut self.wrapper)
            .ok_or(HostError::AlreadyActivatedPlugin)
            .unwrap();

        // PANIC: we dropped the only processor produced, and checked if it matched
        wrapper.deactivate().unwrap();
    }

    #[inline]
    pub fn call_on_main_thread_callback(&mut self) {
        // SAFETY: this is done on the main thread, and the &mut reference guarantees no aliasing
        unsafe { self.wrapper.on_main_thread() }
    }

    #[inline]
    pub fn raw_instance(&self) -> &clap_plugin {
        self.wrapper.raw_instance()
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        // SAFETY: the arc is never cloned
        let wrapper = unsafe { pin_get_ptr(&self.wrapper) };
        Arc::strong_count(wrapper) > 1
    }

    #[inline]
    pub fn shared_host_data(&self) -> &<H as PluginHoster>::Shared {
        self.wrapper.shared()
    }

    #[inline]
    pub fn main_thread_host_data(&self) -> &H {
        // SAFETY: we take &self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.wrapper.main_thread().as_ref() }
    }

    #[inline]
    pub fn main_thread_host_data_mut(&mut self) -> &mut H {
        // SAFETY: we take &mut self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.wrapper.main_thread().as_mut() }
    }

    #[inline]
    pub fn shared_plugin_data(&self) -> PluginSharedHandle {
        PluginSharedHandle::new((self.wrapper.raw_instance() as *const _) as *mut _)
    }

    #[inline]
    pub fn main_thread_plugin_data(&self) -> PluginMainThreadHandle {
        PluginMainThreadHandle::new((self.wrapper.raw_instance() as *const _) as *mut _)
    }
}
