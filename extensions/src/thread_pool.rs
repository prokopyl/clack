//! Allows plugins to use a host's thread pool for multithreaded audio processing.
#![deny(missing_docs)]

use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::thread_pool::*;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

/// Plugin-side of the ThreadPool extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginThreadPool(RawExtension<PluginExtensionSide, clap_plugin_thread_pool>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginThreadPool {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_THREAD_POOL];
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// Host-side of the ThreadPool extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostThreadPool(RawExtension<HostExtensionSide, clap_host_thread_pool>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostThreadPool {
    const IDENTIFIERS: &[&CStr] = &[CLAP_EXT_THREAD_POOL];
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        // SAFETY: the guarantee that this pointer is of the correct type is upheld by the caller.
        Self(unsafe { raw.cast() })
    }
}

/// An error that occurred as a plugin requested access to the host's thread pool.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct ThreadPoolRequestError;

impl Display for ThreadPoolRequestError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to send execution request to the host's thread pool")
    }
}

impl Error for ThreadPoolRequestError {}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::extensions::prelude::*;

    /// Implementation of the Plugin-side of the Thread Pool extension.
    pub trait PluginThreadPoolImpl {
        /// A callback that gets called from the Host's thread pool, for each task the plugin
        /// requested as it called `request_exec`.
        ///
        /// The index of the requested task to execute is given, and must be in the `0..task_count` range.
        fn exec(&self, task_index: u32);
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P> ExtensionImplementation<P> for PluginThreadPool
    where
        for<'a> P: Plugin<Shared<'a>: PluginThreadPoolImpl>,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_thread_pool {
                exec: Some(exec::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn exec<P>(plugin: *const clap_plugin, task_index: u32)
    where
        for<'a> P: Plugin<Shared<'a>: PluginThreadPoolImpl>,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            plugin.shared().exec(task_index);
            Ok(())
        });
    }

    impl HostThreadPool {
        /// Schedules a given number of tasks in the Host's thread pool.
        ///
        /// This will call the plugin's `exec` method a total of `task_count` times, from the Host's thread pool.
        ///
        /// This method will block the current thread until all tasks are processed.
        ///
        /// This method must *only* be called during the `process` method, otherwise
        /// [`ThreadPoolRequestError`] will be returned.
        ///
        /// # Errors
        ///
        /// This method will return [`ThreadPoolRequestError`] if the host denied the request.
        pub fn request_exec(
            &self,
            host: &mut HostAudioProcessorHandle,
            task_count: u32,
        ) -> Result<(), ThreadPoolRequestError> {
            let request_exec = host
                .use_extension(&self.0)
                .request_exec
                .ok_or(ThreadPoolRequestError)?;
            // SAFETY: This type ensures the function pointer is valid.
            let success = unsafe { request_exec(host.as_raw(), task_count) };

            match success {
                true => Ok(()),
                false => Err(ThreadPoolRequestError),
            }
        }
    }
}

#[cfg(feature = "clack-plugin")]
pub use plugin::*;

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_host::extensions::prelude::*;

    /// Implementation of the Host-side of the Thread Pool extension.
    pub trait HostThreadPoolImpl {
        /// Schedules a given number of tasks in the Host's thread pool.
        ///
        /// This will call the plugin's `exec` method a total of `task_count` times, from the Host's thread pool.
        ///
        /// This method will block the current thread until all tasks are processed.
        ///
        /// This method must *only* be called during the `process` method, otherwise
        /// [`ThreadPoolRequestError`] will be returned.
        ///
        /// # Errors
        ///
        /// This method will return an error if the host denied the request.
        fn request_exec(&mut self, task_count: u32) -> Result<(), HostError>;
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<H> ExtensionImplementation<H> for HostThreadPool
    where
        for<'a> H: HostHandlers<AudioProcessor<'a>: HostThreadPoolImpl>,
    {
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_thread_pool {
                request_exec: Some(request_exec::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn request_exec<H>(host: *const clap_host, num_tasks: u32) -> bool
    where
        for<'a> H: HostHandlers<AudioProcessor<'a>: HostThreadPoolImpl>,
    {
        HostWrapper::<H>::handle(host, |host| {
            Ok(host
                .audio_processor()?
                .as_mut()
                .request_exec(num_tasks)
                .is_ok())
        })
        .unwrap_or(false)
    }

    impl PluginThreadPool {
        /// A callback that gets called from the Host's thread pool, for each task the plugin
        /// requested as it called `request_exec`.
        ///
        /// The index of the requested task to execute is given, and must be in the `0..task_count` range.
        pub fn exec(&self, plugin: &PluginSharedHandle, task_index: u32) {
            if let Some(exec) = plugin.use_extension(&self.0).exec {
                // SAFETY: This type ensures the function pointer is valid.
                unsafe { exec(plugin.as_raw_ptr(), task_index) }
            }
        }
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;
