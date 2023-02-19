//! Allows plugins to use an host's thread pool for multi-threaded audio processing.
#![deny(missing_docs)]

use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide};
use clap_sys::ext::thread_pool::*;
use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

/// Plugin-side of the ThreadPool extension.
#[repr(C)]
pub struct PluginThreadPool(clap_plugin_thread_pool);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginThreadPool {}
unsafe impl Sync for PluginThreadPool {}

unsafe impl Extension for PluginThreadPool {
    const IDENTIFIER: &'static CStr = CLAP_EXT_THREAD_POOL;
    type ExtensionSide = PluginExtensionSide;
}

/// Host-side of the ThreadPool extension.
#[repr(C)]
pub struct HostThreadPool(clap_host_thread_pool);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostThreadPool {}
unsafe impl Sync for HostThreadPool {}

unsafe impl Extension for HostThreadPool {
    const IDENTIFIER: &'static CStr = CLAP_EXT_THREAD_POOL;
    type ExtensionSide = HostExtensionSide;
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
    use clap_sys::ext::thread_pool::clap_plugin_thread_pool;

    /// Implementation of the Plugin-side of the Thread Pool extension.
    pub trait PluginThreadPoolImpl {
        /// A callback that gets called from the Host's thread pool, for each task the plugin
        /// requested as it called `request_exec`.
        ///
        /// The index of the requested task to execute is given, and must be in the `0..task_count` range.
        fn exec(&self, task_index: u32);
    }

    impl<P: for<'a> Plugin<'a>> ExtensionImplementation<P> for PluginThreadPool
    where
        for<'a> <P as Plugin<'a>>::Shared: PluginThreadPoolImpl,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: &'static Self = &Self(clap_plugin_thread_pool {
            exec: Some(exec::<P>),
        });
    }

    unsafe extern "C" fn exec<P: for<'a> Plugin<'a>>(plugin: *const clap_plugin, task_index: u32)
    where
        for<'a> <P as Plugin<'a>>::Shared: PluginThreadPoolImpl,
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
            host: &mut HostAudioThreadHandle,
            task_count: u32,
        ) -> Result<(), ThreadPoolRequestError> {
            let request_exec = self.0.request_exec.ok_or(ThreadPoolRequestError)?;
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
        /// This method will return [`ThreadPoolRequestError`] if the host denied the request.
        fn request_exec(&mut self, task_count: u32) -> Result<(), ThreadPoolRequestError>;
    }

    impl<H: for<'a> Host<'a>> ExtensionImplementation<H> for HostThreadPool
    where
        for<'a> <H as Host<'a>>::AudioProcessor: HostThreadPoolImpl,
    {
        const IMPLEMENTATION: &'static Self = &Self(clap_host_thread_pool {
            request_exec: Some(request_exec::<H>),
        });
    }

    unsafe extern "C" fn request_exec<H: for<'a> Host<'a>>(
        host: *const clap_host,
        num_tasks: u32,
    ) -> bool
    where
        for<'a> <H as Host<'a>>::AudioProcessor: HostThreadPoolImpl,
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
            if let Some(exec) = self.0.exec {
                unsafe { exec(plugin.as_raw(), task_index) }
            }
        }
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;
