use crate::host::{PluginHost, PluginHoster};
use clap_sys::plugin::clap_plugin;
use std::ops::RangeInclusive;
use std::pin::Pin;
use std::sync::Arc;

use crate::entry::PluginEntry;
use crate::instance::processor::StoppedPluginAudioProcessor;
use crate::plugin::{PluginMainThread, PluginShared};
use crate::wrapper::{HostError, HostWrapper};

pub struct PluginAudioConfiguration {
    pub sample_rate: f64,
    pub frames_count_range: RangeInclusive<u32>,
}

pub struct PluginInstance<'a, H: PluginHoster<'a>> {
    wrapper: Pin<Arc<HostWrapper<'a, H>>>,
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

impl<'a, H: PluginHoster<'a>> PluginInstance<'a, H> {
    pub fn new<FS, FH>(
        shared: FS,
        hoster: FH,
        entry: &PluginEntry<'a>,
        plugin_id: &[u8],
        host: &PluginHost,
    ) -> Result<Self, HostError>
    where
        FS: FnOnce() -> H::Shared,
        FH: FnOnce(&'a H::Shared) -> H,
    {
        let wrapper = HostWrapper::new(hoster, shared, entry, plugin_id, host.shared().clone())?;

        Ok(Self { wrapper })
    }

    pub fn activate(
        &mut self,
        audio_processor: H::AudioProcessor,
        configuration: PluginAudioConfiguration,
    ) -> Result<StoppedPluginAudioProcessor<'a, H>, HostError> {
        let wrapper =
            arc_get_pin_mut(&mut self.wrapper).ok_or(HostError::AlreadyActivatedPlugin)?;

        wrapper.activate(audio_processor, configuration)?;

        Ok(StoppedPluginAudioProcessor::new(self.wrapper.clone()))
    }

    pub fn deactivate(&mut self, processor: StoppedPluginAudioProcessor<'a, H>) {
        // SAFETY: we never clone the arcs, only compare them
        if unsafe { !Arc::ptr_eq(pin_get_ptr(&self.wrapper), pin_get_ptr(&processor.wrapper)) } {
            panic!("") // TODO
        }

        ::core::mem::drop(processor);

        // PANIC: we dropped the only processor produced, and checked if it matched
        let wrapper = arc_get_pin_mut(&mut self.wrapper)
            .ok_or(HostError::AlreadyActivatedPlugin)
            .unwrap();

        // PANIC: we dropped the only processor produced, and checked if it matched
        wrapper.deactivate().unwrap();
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
    pub fn shared_host_data(&self) -> &H::Shared {
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
        // SAFETY: we take &self, the only reference to the wrapper on the main thread, therefore
        // we can guarantee there are no mutable reference anywhere
        unsafe { self.wrapper.main_thread().as_mut() }
    }

    #[inline]
    pub fn shared_plugin_data(&mut self) -> PluginShared {
        PluginShared::new((self.wrapper.raw_instance() as *const _) as *mut _)
    }

    #[inline]
    pub fn main_thread_plugin_data(&mut self) -> PluginMainThread {
        PluginMainThread::new((self.wrapper.raw_instance() as *const _) as *mut _)
    }
}
