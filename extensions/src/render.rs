#![deny(missing_docs)]

//! Allows plugins to know if their audio processing is running under realtime constraints or not.
//!
//! Plugins that do not implement this extension are considered by host to not care if they are
//! running under realtime constraints or not, and will run just the same either way.
//!
//! If this information does not influence your rendering code, your plugin should **NOT**
//! implement this extension.

use clack_common::extensions::{Extension, PluginExtensionSide};
use clap_sys::ext::render::*;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

/// The Plugin-side of the Render extension.
#[repr(C)]
pub struct PluginRender(clap_plugin_render);

unsafe impl Extension for PluginRender {
    const IDENTIFIER: &'static CStr = CLAP_EXT_RENDER;
    type ExtensionSide = PluginExtensionSide;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[repr(i32)]
/// The different modes of rendering a plugin may be subjected to.
pub enum RenderMode {
    #[default]
    /// Realtime processing, which is the default kind of processing most plugins are expected to operate in.
    Realtime = CLAP_RENDER_REALTIME,
    /// Offline rendering, for processing without realtime pressure (e.g. when exporting/rendering a project or sample).
    ///
    /// In this mode, the plugin may perform allocations on the audio thread, or use more expensive
    /// algorithms for higher sound quality if available.
    Offline = CLAP_RENDER_OFFLINE,
}

impl RenderMode {
    /// Returns the render mode as the raw C-FFI-compatible integer type.
    #[inline]
    pub fn as_raw(&self) -> clap_plugin_render_mode {
        *self as _
    }

    /// Reads the render mode from the raw C-FFI-compatible integer type.
    ///
    /// This may return [`None`] if the given integer's value doesn't match any known render modes.
    #[inline]
    pub fn from_raw(raw_render_mode: clap_plugin_render_mode) -> Option<Self> {
        match raw_render_mode {
            CLAP_RENDER_REALTIME => Some(Self::Realtime),
            CLAP_RENDER_OFFLINE => Some(Self::Offline),
            _ => None,
        }
    }
}

/// An errors that occurs when the plugin either declined or failed to switch to a new render mode.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct PluginRenderError;

impl Display for PluginRenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to set plugin's render mode.")
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::extensions::prelude::*;

    /// Implementation of the Plugin-side of the Render extension.
    pub trait PluginRenderImpl {
        /// Returns `true` if the plugin has an hard requirement to process in real-time.
        ///
        /// This is especially useful for plugins that are acting as a proxy to hardware devices, or
        /// other real-time events.
        fn has_hard_realtime_requirement(&self) -> bool;

        /// Switches the current render mode to the given [`RenderMode`].
        ///
        /// # Errors
        ///
        /// This may return [`PluginRenderError`] if the plugin either declined or failed to switch
        /// to the given render mode.
        fn set(&mut self, mode: RenderMode) -> Result<(), PluginRenderError>;
    }

    impl<P: for<'a> Plugin<'a>> ExtensionImplementation<P> for PluginRender
    where
        for<'a> <P as Plugin<'a>>::MainThread: PluginRenderImpl,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: &'static Self = &Self(clap_plugin_render {
            set: Some(set::<P>),
            has_hard_realtime_requirement: Some(has_hard_realtime_requirement::<P>),
        });
    }

    unsafe extern "C" fn set<P: for<'a> Plugin<'a>>(
        plugin: *const clap_plugin,
        mode: clap_plugin_render_mode,
    ) -> bool
    where
        for<'a> <P as Plugin<'a>>::MainThread: PluginRenderImpl,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            let mode = RenderMode::from_raw(mode).ok_or(PluginWrapperError::InvalidParameter(
                "clap_plugin_render_mode",
            ))?;

            Ok(plugin.main_thread().as_mut().set(mode).is_ok())
        })
        .unwrap_or(false)
    }

    unsafe extern "C" fn has_hard_realtime_requirement<P: for<'a> Plugin<'a>>(
        plugin: *const clap_plugin,
    ) -> bool
    where
        for<'a> <P as Plugin<'a>>::MainThread: PluginRenderImpl,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            Ok(plugin
                .main_thread()
                .as_ref()
                .has_hard_realtime_requirement())
        })
        .unwrap_or(false)
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_host::extensions::prelude::*;

    impl PluginRender {
        /// Returns `true` if the plugin has an hard requirement to process in real-time.
        ///
        /// This is especially useful for plugins that are acting as a proxy to hardware devices, or
        /// other real-time events.
        #[inline]
        pub fn has_realtime_requirement(&self, handle: &mut PluginMainThreadHandle) -> bool {
            if let Some(has_hard_realtime_requirement) = self.0.has_hard_realtime_requirement {
                unsafe { has_hard_realtime_requirement(handle.as_raw()) }
            } else {
                false
            }
        }

        /// Switches the current render mode to the given [`RenderMode`].
        ///
        /// # Errors
        ///
        /// This may return [`PluginRenderError`] if the plugin either declined or failed to switch
        /// to the given render mode.
        pub fn set(
            &self,
            handle: &mut PluginMainThreadHandle,
            render_mode: RenderMode,
        ) -> Result<(), PluginRenderError> {
            let success = unsafe {
                (self.0.set.ok_or(PluginRenderError)?)(handle.as_raw(), render_mode.as_raw())
            };

            match success {
                true => Ok(()),
                false => Err(PluginRenderError),
            }
        }
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;
