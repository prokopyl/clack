//! Utilities to manipulate Plugin instances from an FFI context.
//!
//! These unsafe utilities are targeted at extension implementors. Most `clack-plugin` users do not
//! have to use those utilities to use extensions, see `clack-extensions` instead.

use crate::host::{HostHandle, HostMainThreadHandle};
use crate::plugin::{
    logging, AudioConfiguration, Plugin, PluginError, PluginInstanceImpl, PluginMainThread,
    PluginShared,
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
    #[allow(unused)]
    pub use std::panic::catch_unwind;

    #[cfg(test)]
    #[inline]
    #[allow(unused)]
    pub fn catch_unwind<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
        f: F,
    ) -> std::thread::Result<R> {
        Ok(f())
    }
}

/// A wrapper around a `clack` plugin of a given type.
///
/// This wrapper allows access to a plugin's [`Shared`](crate::plugin::Plugin::Shared),
/// [`MainThread`](crate::plugin::Plugin::MainThread), and Audio Processor (i.e. [`Plugin`](crate::plugin::Plugin)) structs, while
/// also handling common FFI issues, such as error management and unwind safety.
///
/// The only way to access an instance of `PluginWrapper` is through the
/// [`handle`](crate::plugin::wrapper::PluginWrapper::handle) function.
pub struct PluginWrapper<'a, P: Plugin<'a>> {
    host: HostHandle<'a>,
    shared: P::Shared,
    main_thread: UnsafeCell<P::MainThread>,
    audio_processor: Option<UnsafeCell<P>>,
}

impl<'a, P: Plugin<'a>> PluginWrapper<'a, P> {
    pub(crate) fn new(host: HostMainThreadHandle<'a>) -> Result<Self, PluginError> {
        let shared = P::Shared::new(host.shared())?;
        let main_thread = UnsafeCell::new(P::MainThread::new(host, &shared)?);

        Ok(Self {
            host: host.shared(),
            shared,
            main_thread,
            audio_processor: None,
        })
    }

    /// # Safety
    /// Caller must ensure this method is only called on main thread and has exclusivity
    pub(crate) unsafe fn activate(
        self: Pin<&mut Self>,
        audio_config: AudioConfiguration,
    ) -> Result<(), PluginWrapperError> {
        if self.audio_processor.is_some() {
            return Err(PluginWrapperError::ActivatedPlugin);
        }

        // SAFETY: self cannot move, and pointer is valid for the lifetime of P
        let shared = &*(&self.shared as *const _);
        let host = self.host;
        // SAFETY: we only update the fields, we don't move the struct
        let pinned_self = Pin::get_unchecked_mut(self);

        let processor = P::activate(
            host.to_audio_thread(),
            pinned_self.main_thread.get_mut(),
            shared,
            audio_config,
        )?;
        pinned_self.audio_processor = Some(UnsafeCell::new(processor));

        Ok(())
    }

    /// # Safety
    /// Caller must ensure this method is only called on main thread, and has exclusivity on it
    pub(crate) unsafe fn deactivate(self: Pin<&mut Self>) -> Result<(), PluginWrapperError> {
        // SAFETY: taking the audio processor does not move the whole struct
        let pinned_self = Pin::get_unchecked_mut(self);
        let audio_processor = &mut pinned_self.audio_processor;

        match audio_processor.take() {
            None => Err(PluginWrapperError::DeactivatedPlugin),
            Some(audio_processor) => {
                audio_processor
                    .into_inner()
                    .deactivate(pinned_self.main_thread.get_mut());

                Ok(())
            }
        }
    }

    /// Returns a reference to a plugin's [`Shared`](crate::plugin::Plugin::Shared) struct.
    ///
    /// This is always safe to call in any context, since the `Shared` struct is required to
    /// implement `Sync`.
    #[inline]
    pub fn shared(&self) -> &P::Shared {
        &self.shared
    }

    /// Returns a raw, non-null pointer to the plugin's [`MainThread`](crate::plugin::Plugin::MainThread)
    /// struct.
    ///
    /// # Safety
    /// The caller must ensure this method is only called on the main thread.
    ///
    /// The pointer is safe to mutably dereference, as long as the caller ensures it is not being
    /// aliased, as per usual safety rules.
    #[inline]
    pub unsafe fn main_thread(&self) -> NonNull<P::MainThread> {
        // SAFETY: pointer has been created from reference, it cannot be null.
        NonNull::new_unchecked(self.main_thread.get())
    }

    /// Returns a raw, non-null pointer to the plugin's audio processor
    /// (i.e. [`Plugin`](crate::plugin::Plugin)) struct.
    ///
    /// # Errors
    ///
    /// This method will return `PluginWrapperError::DeactivatedPlugin` if the plugin has not been
    /// activated before calling this method.
    ///
    /// This is an extra safety check which ensures that hosts correctly activated plugins before
    /// calling any audio-thread method.
    ///
    /// # Safety
    /// The caller must ensure this method is only called on the audio thread.
    ///
    /// The pointer is safe to mutably dereference, as long as the caller ensures it is not being
    /// aliased, as per usual safety rules.
    #[inline]
    pub unsafe fn audio_processor(&self) -> Result<NonNull<P>, PluginWrapperError> {
        self.audio_processor
            .as_ref()
            // SAFETY: pointer has been created from reference, it cannot be null.
            .map(|p| NonNull::new_unchecked(p.get()))
            .ok_or(PluginWrapperError::DeactivatedPlugin)
    }

    /// Provides a shared reference to a plugin wrapper of a given type, to the given handler
    /// closure.
    ///
    /// Besides providing a reference, this function does a few extra safety checks:
    ///
    /// * The given `clap_plugin` pointer is null-checked, as well as some other host-provided
    /// pointers;
    /// * The handler is wrapped in [`std::panic::catch_unwind`];
    /// * Any [`PluginWrapperError`] returned by the handler is caught.
    ///
    /// If any of the above safety check fails, an error message is logged (using the standard CLAP
    /// logging extension). If logging is unavailable or fails for any reason, the error message is
    /// written to `stderr` as a fallback.
    ///
    /// Note that some safety checks (e.g. the `clap_plugin` pointer null-checks) may result in the
    /// closure never being called, and an error being returned only. Users of this function must
    /// not rely on the completion of this closure for safety, and must handle this function
    /// returning `None` gracefully.
    ///
    /// If all goes well, the return value of the handler closure is forwarded and returned by this
    /// function.
    ///
    /// # Errors
    /// If any safety check failed, or any error or panic occurred inside the handler closure, this
    /// function returns `None`, and the error message is logged.
    ///
    /// # Safety
    ///
    /// The given plugin type `P` **must** be the correct type for the received pointer. Otherwise,
    /// incorrect casts will occur, which will lead to Undefined Behavior.
    ///
    /// The `plugin` pointer must also point to a valid instance of `clap_plugin`, as provided by
    /// the CLAP Host. While this function does a couple of simple safety checks, only a few common
    /// cases are actually covered (i.e. null checks), and those **must not** be relied upon: those
    /// checks only exist to help debugging faulty hosts.
    ///
    /// # Example
    ///
    /// This is the implementation of the [`on_main_thread`](crate::plugin::PluginMainThread::on_main_thread)
    /// callback's C wrapper.
    ///
    /// This method is guaranteed by the CLAP specification to be only called on the main thread.
    ///
    /// ```
    /// use clap_sys::plugin::clap_plugin;
    /// use clack_plugin::plugin::{Plugin, PluginMainThread, wrapper::PluginWrapper};
    ///
    /// unsafe extern "C" fn on_main_thread<'a, P: Plugin<'a>>(plugin: *const clap_plugin) {
    ///   PluginWrapper::<P>::handle(plugin, |p| {
    ///     p.main_thread().as_mut().on_main_thread();
    ///     Ok(())
    ///   });
    /// }
    /// ```
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

    unsafe fn from_raw(raw: *const clap_plugin) -> Result<&'a Self, PluginWrapperError> {
        raw.as_ref()
            .ok_or(PluginWrapperError::NullPluginInstance)?
            .plugin_data
            .cast::<PluginInstanceImpl<'a, P>>()
            .as_ref()
            .ok_or(PluginWrapperError::NullPluginData)?
            .plugin_data
            .as_ref()
            .ok_or(PluginWrapperError::UninitializedPlugin)
    }

    unsafe fn from_raw_mut(
        raw: *const clap_plugin,
    ) -> Result<Pin<&'a mut Self>, PluginWrapperError> {
        Ok(Pin::new_unchecked(
            raw.as_ref()
                .ok_or(PluginWrapperError::NullPluginInstance)?
                .plugin_data
                .cast::<PluginInstanceImpl<'a, P>>()
                .as_mut()
                .ok_or(PluginWrapperError::NullPluginData)?
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

/// Errors raised by a [`PluginWrapper`].
#[derive(Debug)]
pub enum PluginWrapperError {
    /// The `clap_plugin` raw pointer was null.
    NullPluginInstance,
    /// The `clap_plugin.plugin_data` raw pointer was null.
    NullPluginData,
    /// A unexpectedly null raw pointer was encountered.
    ///
    /// The given string may contain more information about which pointer was found to be null.
    NulPtr(&'static str),
    /// The plugin was not properly initialized (i.e. `init` was not called or had failed).
    UninitializedPlugin,
    /// An attempt was made to call `activate` on an already activated plugin.
    ActivatedPlugin,
    /// An attempt was made to call an audio-thread function while the plugin was deactivated
    /// (e.g. without previously calling `activate`).
    DeactivatedPlugin,
    /// The plugin panicked during a function call.
    Panic,
    /// A given [`PluginError`](crate::plugin::PluginError) was raised during a function call.
    Plugin(PluginError),
    /// A generic or custom error of a given severity.
    Any(clap_log_severity, Box<dyn Error>),
}

impl PluginWrapperError {
    /// Returns the severity of this error.
    ///
    /// This is mainly useful for logging.
    ///
    /// # Example
    ///
    /// ```
    /// # use clap_sys::ext::log::CLAP_LOG_PLUGIN_MISBEHAVING;
    /// use clack_plugin::plugin::wrapper::PluginWrapperError;
    /// let error = PluginWrapperError::Panic;
    ///
    /// assert_eq!(error.severity(), CLAP_LOG_PLUGIN_MISBEHAVING);
    /// ```
    pub fn severity(&self) -> clap_log_severity {
        match self {
            PluginWrapperError::Plugin(_) => CLAP_LOG_ERROR,
            PluginWrapperError::Panic => CLAP_LOG_PLUGIN_MISBEHAVING,
            PluginWrapperError::Any(s, _) => *s,
            _ => CLAP_LOG_HOST_MISBEHAVING,
        }
    }

    /// Returns a closure that maps an error to a [`PluginWrapperError::Any`] error of a given
    /// severity.
    ///
    /// This is an useful helper method when paired with [`Result::map_err`].
    ///
    /// # Example
    /// ```
    /// use clap_sys::ext::log::CLAP_LOG_PLUGIN_MISBEHAVING;
    /// # use clack_plugin::plugin::wrapper::PluginWrapperError;
    ///
    /// let x: Result<(), _> = Err(std::env::VarError::NotPresent); // Some random error type
    /// let clap_error = x.map_err(PluginWrapperError::with_severity(CLAP_LOG_PLUGIN_MISBEHAVING));
    ///
    /// assert_eq!(clap_error.unwrap_err().severity(), CLAP_LOG_PLUGIN_MISBEHAVING);
    /// ```
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
            PluginWrapperError::NullPluginInstance => {
                f.write_str("Plugin method was called with null clap_plugin pointer")
            }
            PluginWrapperError::NullPluginData => {
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
                f.write_str("Plugin was not activated before calling a audio-thread method")
            }
            PluginWrapperError::Plugin(e) => std::fmt::Display::fmt(&e, f),
            PluginWrapperError::Any(_, e) => std::fmt::Display::fmt(e, f),
            PluginWrapperError::Panic => f.write_str("Plugin panicked"),
        }
    }
}

impl Error for PluginWrapperError {}
