#![deny(missing_docs)]

//! Host-driven Timer support.
//!
//! This extension allows plugins to register timers to the host, which will then proceed to call
//! a plugin's callback at a given regular interval.

use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide};
use clap_sys::ext::timer_support::*;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

/// Host-side of the Timer extension.
#[repr(C)]
pub struct HostTimer(clap_host_timer_support);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostTimer {}
unsafe impl Sync for HostTimer {}

unsafe impl Extension for HostTimer {
    const IDENTIFIER: &'static CStr = CLAP_EXT_TIMER_SUPPORT;
    type ExtensionSide = HostExtensionSide;
}

/// Plugin-side of the Timer extension.
#[repr(C)]
pub struct PluginTimer(clap_plugin_timer_support);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginTimer {}
unsafe impl Sync for PluginTimer {}

unsafe impl Extension for PluginTimer {
    const IDENTIFIER: &'static CStr = CLAP_EXT_TIMER_SUPPORT;
    type ExtensionSide = PluginExtensionSide;
}

/// An identifier representing a timer given to a plugin.
///
/// Each identifier must be unique for a specific plugin instance.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct TimerId(pub u32);

/// Errors that can occur while setting up Timers.
// TODO: make global Clack error type all of these can be turned into
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum TimerError {
    /// The host failed or declined to register a timer.
    RegisterError,
    /// The host failed to unregister a timer.
    UnregisterError,
}

impl Display for TimerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerError::RegisterError => f.write_str("Failed to register CLAP Timer"),
            TimerError::UnregisterError => f.write_str("Failed to unregister CLAP Timer"),
        }
    }
}

impl Error for TimerError {}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::extensions::prelude::*;

    impl HostTimer {
        /// Registers a new Timer, returning its unique [`TimerId`].
        ///
        /// The host will then proceed to call the plugin's `on_timer` callback for every tick of
        /// this new Timer.
        ///
        /// Note the Host is allowed to adjust the period if it's too short, under a certain threshold.
        /// In general, at least 30Hz should be allowed by every host, although this is not a hard requirement.
        ///
        /// # Errors
        ///
        /// Returns [`TimerError::RegisterError`] if the host failed or denied to register this timer.
        #[inline]
        pub fn register_timer(
            &self,
            host: &HostHandle,
            period_ms: u32,
        ) -> Result<TimerId, TimerError> {
            let mut id = 0u32;
            let register_timer = self.0.register_timer.ok_or(TimerError::RegisterError)?;

            match unsafe { register_timer(host.as_raw(), period_ms, &mut id) } {
                true => Ok(TimerId(id)),
                false => Err(TimerError::RegisterError),
            }
        }

        /// Unregisters a given Timer, identified by its unique [`TimerId`].
        ///
        /// After this call, the host will no longer call the plugin's `on_timer` callback for every tick of
        /// the given Timer.
        ///
        /// # Errors
        ///
        /// Returns [`TimerError::UnregisterError`] if the host failed to unregister this timer.
        #[inline]
        pub fn unregister_timer(
            &self,
            host: &HostHandle,
            timer_id: TimerId,
        ) -> Result<(), TimerError> {
            let unregister_timer = self.0.unregister_timer.ok_or(TimerError::UnregisterError)?;

            match unsafe { unregister_timer(host.as_raw(), timer_id.0) } {
                true => Ok(()),
                false => Err(TimerError::RegisterError),
            }
        }
    }

    /// Implementation of the Plugin-side of the Timer extension.
    pub trait PluginTimerImpl {
        /// A callback that gets called every time a Timer registered by this plugin ticks.
        ///
        /// The callback is also given the unique [`TimerId`] of the timer that ticked and triggered
        /// it.
        fn on_timer(&mut self, timer_id: TimerId);
    }

    impl<P: Plugin> ExtensionImplementation<P> for PluginTimer
    where
        for<'a> P::MainThread<'a>: PluginTimerImpl,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: &'static Self = &PluginTimer(clap_plugin_timer_support {
            on_timer: Some(on_timer::<P>),
        });
    }

    unsafe extern "C" fn on_timer<P: Plugin>(plugin: *const clap_plugin, timer_id: u32)
    where
        for<'a> P::MainThread<'a>: PluginTimerImpl,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            plugin.main_thread().as_mut().on_timer(TimerId(timer_id));
            Ok(())
        });
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_host::extensions::prelude::*;

    /// Implementation of the Host-side of the Timer extension.
    pub trait HostTimerImpl {
        /// Registers a new Timer, returning its unique [`TimerId`].
        ///
        /// The host then needs to call the plugin's `on_timer` callback for every tick of
        /// this new Timer.
        ///
        /// Note the Host is allowed to adjust the period if it's too short, under a certain threshold.
        /// In general, at least 30Hz should be allowed by every host, although this is not a hard requirement.
        ///
        /// # Errors
        ///
        /// Returns [`TimerError::RegisterError`] if the host failed or denied to register this timer.
        fn register_timer(&mut self, period_ms: u32) -> Result<TimerId, TimerError>;

        /// Unregisters a given Timer, identified by its unique [`TimerId`].
        ///
        /// After this call, the host will no longer call the plugin's `on_timer` callback for every tick of
        /// the given Timer.
        ///
        /// # Errors
        ///
        /// Returns [`TimerError::UnregisterError`] if the host failed to unregister this timer.
        fn unregister_timer(&mut self, timer_id: TimerId) -> Result<(), TimerError>;
    }

    impl<H: Host> ExtensionImplementation<H> for HostTimer
    where
        for<'a> <H as Host>::MainThread<'a>: HostTimerImpl,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: &'static Self = &HostTimer(clap_host_timer_support {
            register_timer: Some(register_timer::<H>),
            unregister_timer: Some(unregister_timer::<H>),
        });
    }

    unsafe extern "C" fn register_timer<H: Host>(
        host: *const clap_host,
        period_ms: u32,
        timer_id: *mut u32,
    ) -> bool
    where
        for<'a> <H as Host>::MainThread<'a>: HostTimerImpl,
    {
        HostWrapper::<H>::handle(host, |host| {
            match host.main_thread().as_mut().register_timer(period_ms) {
                Ok(id) => {
                    *timer_id = id.0;
                    Ok(true)
                }
                Err(_) => {
                    *timer_id = u32::MAX;
                    Ok(false)
                }
            }
        })
        .unwrap_or(false)
    }

    unsafe extern "C" fn unregister_timer<H: Host>(host: *const clap_host, timer_id: u32) -> bool
    where
        for<'a> <H as Host>::MainThread<'a>: HostTimerImpl,
    {
        HostWrapper::<H>::handle(host, |host| {
            Ok(host
                .main_thread()
                .as_mut()
                .unregister_timer(TimerId(timer_id))
                .is_ok())
        })
        .unwrap_or(false)
    }

    impl PluginTimer {
        /// A callback that gets called every time a Timer registered by this plugin ticks.
        ///
        /// The callback is also given the unique [`TimerId`] of the timer that ticked and triggered
        /// it.
        #[inline]
        pub fn on_timer(&self, plugin: &mut PluginMainThreadHandle, timer_id: TimerId) {
            if let Some(on_timer) = self.0.on_timer {
                unsafe { on_timer(plugin.as_raw(), timer_id.0) }
            }
        }
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;
