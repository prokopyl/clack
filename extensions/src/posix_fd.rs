//! Allows plugins to hook themselves into the host's main thread event reactor.
//!
//! This is useful to handle asynchronous I/O on the main thread.

#![deny(missing_docs)]

use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtensionSide, PluginExtensionSide, RawExtension};
use clap_sys::ext::posix_fd_support::*;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::os::unix::io::RawFd;

bitflags! {
    /// IO events flags for file descriptors.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FdFlags: u32 {
        /// A read event
        const READ = CLAP_POSIX_FD_READ;
        /// A write event
        const WRITE = CLAP_POSIX_FD_WRITE;
        /// An error event
        const ERROR = CLAP_POSIX_FD_ERROR;
    }
}

/// Plugin-side of the POSIX File Descriptors extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct PluginPosixFd(RawExtension<PluginExtensionSide, clap_plugin_posix_fd_support>);

/// Plugin-side of the POSIX File Descriptors extension.
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostPosixFd(RawExtension<HostExtensionSide, clap_host_posix_fd_support>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for PluginPosixFd {
    const IDENTIFIER: &'static CStr = CLAP_EXT_POSIX_FD_SUPPORT;
    type ExtensionSide = PluginExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostPosixFd {
    const IDENTIFIER: &'static CStr = CLAP_EXT_POSIX_FD_SUPPORT;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

/// Errors that can occur with the POSIX File Descriptors extension.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FdError {
    /// An error occurred while the plugin tried to register a given [`RawFd`] with the given [`FdFlags`]
    Register((RawFd, FdFlags)),
    /// An error occurred while the plugin tried to modify a given [`RawFd`] with the given [`FdFlags`]
    Modify((RawFd, FdFlags)),
    /// An error occurred while the plugin tried to unregister a given [`RawFd`]
    Unregister(RawFd),
}

impl Display for FdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FdError::Register((fd, flags)) => write!(
                f,
                "Failed to register file descriptor ({fd}) with flags ({flags:?})"
            ),
            FdError::Modify((fd, flags)) => write!(
                f,
                "Failed to modify file descriptor ({fd}) with flags ({flags:?})"
            ),
            FdError::Unregister(fd) => write!(f, "Failed to unregister file descriptor ({fd})"),
        }
    }
}

#[cfg(feature = "clack-host")]
mod host {
    use super::*;
    use clack_host::extensions::prelude::*;

    impl PluginPosixFd {
        /// A callback that gets called for every event on each registered File Descriptor.
        ///
        /// Note this callback is "level-triggered". It means that for instance, a writable File
        /// Descriptor will continuously produce "on_fd()" events.
        #[inline]
        pub fn on_fd(&self, plugin: &mut PluginMainThreadHandle, fd: RawFd, flags: FdFlags) {
            if let Some(on_fd) = plugin.use_extension(&self.0).on_fd {
                // SAFETY: This type ensures the function pointer is valid.
                unsafe { on_fd(plugin.as_raw(), fd, flags.bits()) }
            }
        }
    }

    /// Implementation of the Host-side of the POSIX File Descriptors extension.
    pub trait HostPosixFdImpl {
        /// Registers a given File Descriptor into the host's event reactor, for a given set of events.
        ///
        /// The host will call the plugin's `on_fd` method every time the File Descriptor fires one
        /// of these events.
        fn register_fd(&mut self, fd: RawFd, flags: FdFlags) -> Result<(), HostError>;
        /// Updates the set of events a given File Descriptor will fire.
        fn modify_fd(&mut self, fd: RawFd, flags: FdFlags) -> Result<(), HostError>;
        /// Removes a given File Descriptor from the host's event reactor.
        fn unregister_fd(&mut self, fd: RawFd) -> Result<(), HostError>;
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<H> ExtensionImplementation<H> for HostPosixFd
    where
        H: HostHandlers<MainThread: HostPosixFdImpl>,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_host_posix_fd_support {
                register_fd: Some(register_fd::<H>),
                modify_fd: Some(modify_fd::<H>),
                unregister_fd: Some(unregister_fd::<H>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn register_fd<H: HostHandlers>(
        host: *const clap_host,
        fd: i32,
        flags: clap_posix_fd_flags,
    ) -> bool
    where
        for<'a> <H as HostHandlers>::MainThread<'a>: HostPosixFdImpl,
    {
        HostWrapper::<H>::handle(host, |host| {
            Ok(host
                .main_thread()
                .as_mut()
                .register_fd(fd, FdFlags::from_bits_truncate(flags))
                .is_ok())
        })
        .unwrap_or(false)
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn modify_fd<H: HostHandlers>(
        host: *const clap_host,
        fd: i32,
        flags: clap_posix_fd_flags,
    ) -> bool
    where
        for<'a> <H as HostHandlers>::MainThread<'a>: HostPosixFdImpl,
    {
        HostWrapper::<H>::handle(host, |host| {
            Ok(host
                .main_thread()
                .as_mut()
                .modify_fd(fd, FdFlags::from_bits_truncate(flags))
                .is_ok())
        })
        .unwrap_or(false)
    }
    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn unregister_fd<H: HostHandlers>(host: *const clap_host, fd: i32) -> bool
    where
        for<'a> <H as HostHandlers>::MainThread<'a>: HostPosixFdImpl,
    {
        HostWrapper::<H>::handle(host, |host| {
            Ok(host.main_thread().as_mut().unregister_fd(fd).is_ok())
        })
        .unwrap_or(false)
    }
}

#[cfg(feature = "clack-host")]
pub use host::*;

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::extensions::prelude::*;

    impl HostPosixFd {
        /// Registers a given File Descriptor into the host's event reactor, for a given set of events.
        ///
        /// The host will call the plugin's `on_fd` method every time the File Descriptor fires one
        /// of these events.
        pub fn register_fd(
            &self,
            host: &mut HostMainThreadHandle,
            fd: RawFd,
            flags: FdFlags,
        ) -> Result<(), FdError> {
            let register_fd = host
                .use_extension(&self.0)
                .register_fd
                .ok_or(FdError::Register((fd, flags)))?;

            // SAFETY: This type ensures the function pointer is valid.
            let success = unsafe { register_fd(host.as_raw(), fd, flags.bits()) };
            match success {
                true => Ok(()),
                false => Err(FdError::Register((fd, flags))),
            }
        }

        /// Updates the set of events a given File Descriptor will fire.
        pub fn modify_fd(
            &self,
            host: &mut HostMainThreadHandle,
            fd: RawFd,
            flags: FdFlags,
        ) -> Result<(), FdError> {
            let modify_fd = host
                .use_extension(&self.0)
                .modify_fd
                .ok_or(FdError::Modify((fd, flags)))?;

            // SAFETY: This type ensures the function pointer is valid.
            let success = unsafe { modify_fd(host.as_raw(), fd, flags.bits()) };
            match success {
                true => Ok(()),
                false => Err(FdError::Modify((fd, flags))),
            }
        }

        /// Removes a given File Descriptor from the host's event reactor.
        pub fn unregister_fd(
            &self,
            host: &mut HostMainThreadHandle,
            fd: RawFd,
        ) -> Result<(), FdError> {
            let unregister_fd = host
                .use_extension(&self.0)
                .unregister_fd
                .ok_or(FdError::Unregister(fd))?;

            // SAFETY: This type ensures the function pointer is valid.
            let success = unsafe { unregister_fd(host.as_raw(), fd) };
            match success {
                true => Ok(()),
                false => Err(FdError::Unregister(fd)),
            }
        }
    }

    /// Implementation of the Plugin-side of the POSIX File Descriptors extension.
    pub trait PluginPosixFdImpl {
        /// A callback that gets called for every event on each registered File Descriptor.
        ///
        /// Note this callback is "level-triggered". It means that for instance, a writable File
        /// Descriptor will continuously produce "on_fd()" events.
        ///
        /// Don't forget to use the `modify_fd` method to remove the write notification once you're
        /// done writing.
        fn on_fd(&mut self, fd: RawFd, flags: FdFlags);
    }

    // SAFETY: The given struct is the CLAP extension struct for the matching side of this extension.
    unsafe impl<P: Plugin> ExtensionImplementation<P> for PluginPosixFd
    where
        for<'a> P::MainThread<'a>: PluginPosixFdImpl,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: RawExtensionImplementation =
            RawExtensionImplementation::new(&clap_plugin_posix_fd_support {
                on_fd: Some(on_fd::<P>),
            });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn on_fd<P: Plugin>(
        plugin: *const clap_plugin,
        fd: i32,
        flags: clap_posix_fd_flags,
    ) where
        for<'a> P::MainThread<'a>: PluginPosixFdImpl,
    {
        PluginWrapper::<P>::handle(plugin, |plugin| {
            plugin
                .main_thread()
                .as_mut()
                .on_fd(fd, FdFlags::from_bits_truncate(flags));

            Ok(())
        });
    }
}
#[cfg(feature = "clack-plugin")]
pub use plugin::*;
