use crate::host::HostHandle;
use crate::plugin::{
    logging, Plugin, PluginError, PluginInstanceImpl, PluginMainThread, PluginShared, SampleConfig,
};
use clap_sys::ext::log::*;
use clap_sys::plugin::clap_plugin;
use std::cell::UnsafeCell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::ptr::NonNull;

mod panic {
    #[cfg(not(test))]
    pub use std::panic::catch_unwind;

    #[cfg(test)]
    #[inline]
    pub fn catch_unwind<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
        f: F,
    ) -> std::thread::Result<R> {
        Ok(f())
    }
}

pub struct PluginWrapper<'a, P: Plugin<'a>> {
    shared: P::Shared,
    main_thread: UnsafeCell<P::MainThread>,
    audio_processor: Option<UnsafeCell<P>>,
}

impl<'a, P: Plugin<'a>> PluginWrapper<'a, P> {
    pub(crate) fn new(host: HostHandle<'a>) -> Result<Self, PluginError> {
        let shared = P::Shared::new(host)?;
        let main_thread = UnsafeCell::new(P::MainThread::new(host, &shared)?);

        Ok(Self {
            shared,
            main_thread,
            audio_processor: None,
        })
    }

    #[inline]
    pub fn shared(&self) -> &P::Shared {
        &self.shared
    }

    /// # Safety
    /// Caller must ensure this method is only called on main thread and has exclusivity
    pub(crate) unsafe fn activate(
        self: Pin<&mut Self>,
        host: HostHandle<'a>,
        sample_config: SampleConfig,
    ) -> Result<(), PluginWrapperError> {
        if self.audio_processor.is_some() {
            return Err(PluginWrapperError::ActivatedPlugin);
        }

        // SAFETY: self cannot move, and pointer is valid for the lifetime of P
        let shared = &*(&self.shared as *const _);
        // SAFETY: we only update the fields, we don't move the struct
        let pinned_self = Pin::get_unchecked_mut(self);

        let processor = P::new(
            host,
            pinned_self.main_thread.get_mut(),
            shared,
            sample_config,
        )?;
        pinned_self.audio_processor = Some(UnsafeCell::new(processor));

        Ok(())
    }

    /// # Safety
    /// Caller must ensure this method is only called on main thread, and has exclusivity on it
    pub(crate) unsafe fn deactivate(self: Pin<&mut Self>) -> Result<(), PluginWrapperError> {
        // SAFETY: taking the audio processor does not move the whole sturct
        let audio_processor = &mut Pin::get_unchecked_mut(self).audio_processor;
        if audio_processor.take().is_none() {
            return Err(PluginWrapperError::DeactivatedPlugin);
        }

        Ok(())
    }

    /// # Safety
    /// Caller must ensure this method is only called on main thread, and has exclusivity on it
    #[inline]
    pub unsafe fn main_thread(&self) -> NonNull<P::MainThread> {
        // SAFETY: pointer has been created from reference
        NonNull::new_unchecked(self.main_thread.get())
    }

    /// # Safety
    /// Caller must ensure this method is only called on an audio thread, and has exclusivity on it
    #[inline]
    pub unsafe fn audio_processor(&self) -> Result<NonNull<P>, PluginWrapperError> {
        self.audio_processor
            .as_ref()
            // SAFETY: pointer has been created from reference
            .map(|p| NonNull::new_unchecked(p.get()))
            .ok_or(PluginWrapperError::DeactivatedPlugin)
    }

    /// # Safety
    /// The plugin pointer must be valid
    pub(crate) unsafe fn handle_plugin_mut<T, F>(
        plugin: *const clap_plugin,
        handler: F,
    ) -> Option<T>
    where
        F: FnOnce(Pin<&mut PluginWrapper<'a, P>>) -> Result<T, PluginWrapperError>,
    {
        match Self::handle_panic_mut(plugin, handler) {
            Ok(value) => Some(value),
            Err(e) => {
                logging::plugin_log::<P>(plugin, &e);

                None
            }
        }
    }

    /// # Safety
    /// The plugin pointer must be valid
    pub unsafe fn handle<T, F>(plugin: *const clap_plugin, handler: F) -> Option<T>
    where
        F: FnOnce(&PluginWrapper<'a, P>) -> Result<T, PluginWrapperError>,
    {
        match Self::handle_panic(plugin, handler) {
            Ok(value) => Some(value),
            Err(e) => {
                logging::plugin_log::<P>(plugin, &e);

                None
            }
        }
    }

    unsafe fn from_raw(raw: *const clap_plugin) -> Result<&'a Self, PluginWrapperError> {
        raw.as_ref()
            .ok_or(PluginWrapperError::NulPluginDesc)?
            .plugin_data
            .cast::<PluginInstanceImpl<'a, P>>()
            .as_ref()
            .ok_or(PluginWrapperError::NulPluginData)?
            .plugin_data
            .as_ref()
            .ok_or(PluginWrapperError::UninitializedPlugin)
    }

    unsafe fn from_raw_mut(
        raw: *const clap_plugin,
    ) -> Result<Pin<&'a mut Self>, PluginWrapperError> {
        Ok(Pin::new_unchecked(
            raw.as_ref()
                .ok_or(PluginWrapperError::NulPluginDesc)?
                .plugin_data
                .cast::<PluginInstanceImpl<'a, P>>()
                .as_mut()
                .ok_or(PluginWrapperError::NulPluginData)?
                .plugin_data
                .as_mut()
                .ok_or(PluginWrapperError::UninitializedPlugin)?,
        ))
    }

    unsafe fn handle_panic<T, F>(
        plugin: *const clap_plugin,
        handler: F,
    ) -> Result<T, PluginWrapperError>
    where
        F: FnOnce(&PluginWrapper<'a, P>) -> Result<T, PluginWrapperError>,
    {
        let plugin = Self::from_raw(plugin)?;

        panic::catch_unwind(AssertUnwindSafe(|| handler(plugin)))
            .map_err(|_| PluginWrapperError::Panic)?
    }

    unsafe fn handle_panic_mut<T, F>(
        plugin: *const clap_plugin,
        handler: F,
    ) -> Result<T, PluginWrapperError>
    where
        F: FnOnce(Pin<&mut PluginWrapper<'a, P>>) -> Result<T, PluginWrapperError>,
    {
        let plugin = Self::from_raw_mut(plugin)?;

        panic::catch_unwind(AssertUnwindSafe(|| handler(plugin)))
            .map_err(|_| PluginWrapperError::Panic)?
    }
}

unsafe impl<'a, P: Plugin<'a>> Send for PluginWrapper<'a, P> {}
unsafe impl<'a, P: Plugin<'a>> Sync for PluginWrapper<'a, P> {}

#[derive(Debug)]
pub enum PluginWrapperError {
    NulPluginDesc,
    NulPluginData,
    NulPtr(&'static str),
    UninitializedPlugin,
    ActivatedPlugin,
    DeactivatedPlugin,
    Panic,
    Plugin(PluginError),
    Any(clap_log_severity, Box<dyn Error>),
}

impl PluginWrapperError {
    pub fn severity(&self) -> clap_log_severity {
        match self {
            PluginWrapperError::Plugin(_) => CLAP_LOG_ERROR,
            PluginWrapperError::Panic => CLAP_LOG_PLUGIN_MISBEHAVING,
            PluginWrapperError::Any(s, _) => *s,
            _ => CLAP_LOG_HOST_MISBEHAVING,
        }
    }

    #[inline]
    pub fn with_severity<E: 'static + Error>(
        severity: clap_log_severity,
    ) -> impl Fn(E) -> PluginWrapperError {
        move |e| PluginWrapperError::Any(severity, Box::new(e))
    }
}

impl From<PluginError> for PluginWrapperError {
    #[inline]
    fn from(e: PluginError) -> Self {
        PluginWrapperError::Plugin(e)
    }
}

impl Display for PluginWrapperError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginWrapperError::NulPluginDesc => {
                f.write_str("Plugin method was called with null clap_plugin pointer")
            }
            PluginWrapperError::NulPluginData => {
                f.write_str("Plugin method was called with null clap_plugin.plugin_data pointer")
            }
            PluginWrapperError::NulPtr(ptr_name) => {
                write!(f, "Plugin method was called with null {} pointer", ptr_name)
            }
            PluginWrapperError::UninitializedPlugin => {
                f.write_str("Plugin was not properly initialized before use")
            }
            PluginWrapperError::ActivatedPlugin => f.write_str("Plugin was already activated"),
            PluginWrapperError::DeactivatedPlugin => {
                f.write_str("Plugin was not activated before calling a processing-thread method")
            }
            PluginWrapperError::Plugin(e) => std::fmt::Display::fmt(&e, f),
            PluginWrapperError::Any(_, e) => std::fmt::Display::fmt(e, f),
            PluginWrapperError::Panic => f.write_str("Plugin panicked"),
        }
    }
}

impl Error for PluginWrapperError {}
