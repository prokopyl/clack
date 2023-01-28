//! Allows plugins to hook themselves into the host's main thread event reactor.
//!
//! This is useful to handle asynchronous I/O on the main thread.

#![deny(missing_docs)]

use bitflags::bitflags;
use clack_common::extensions::{Extension, HostExtension, PluginExtension};
use clap_sys::ext::posix_fd_support::*;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::os::unix::io::RawFd;

bitflags! {
    /// IO events flags for file descriptors.
    #[repr(C)]
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
#[repr(C)]
pub struct PluginPosixFd(clap_plugin_posix_fd_support);

/// Plugin-side of the POSIX File Descriptors extension.
#[repr(C)]
pub struct HostPosixFd(clap_host_posix_fd_support);

unsafe impl Extension for PluginPosixFd {
    const IDENTIFIER: &'static CStr = CLAP_EXT_POSIX_FD_SUPPORT;
    type ExtensionType = PluginExtension;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for PluginPosixFd {}
unsafe impl Sync for PluginPosixFd {}

unsafe impl Extension for HostPosixFd {
    const IDENTIFIER: &'static CStr = CLAP_EXT_POSIX_FD_SUPPORT;
    type ExtensionType = HostExtension;
}

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostPosixFd {}
unsafe impl Sync for HostPosixFd {}

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
    use clack_common::extensions::ExtensionImplementation;
    use clack_host::host::Host;
    use clack_host::plugin::PluginMainThreadHandle;
    use clack_host::wrapper::HostWrapper;
    use clap_sys::host::clap_host;
    use std::os::unix::prelude::RawFd;

    impl PluginPosixFd {
        /// A callback that gets called for every event on each registered File Descriptor.
        ///
        /// Note this callback is "level-triggered". It means that for instance, a writable File
        /// Descriptor will continuously produce "on_fd()" events.
        #[inline]
        pub fn on_fd(&self, plugin: &mut PluginMainThreadHandle, fd: RawFd, flags: FdFlags) {
            if let Some(on_fd) = self.0.on_fd {
                unsafe { on_fd(plugin.as_raw(), fd, flags.bits) }
            }
        }
    }

    /// Implementation of the Host-side of the POSIX File Descriptors extension.
    pub trait HostPosixFdImplementation {
        /// Registers a given File Descriptor into the host's event reactor, for a given set of events.
        ///
        /// The host will call the plugin's `on_fd` method every time the File Descriptor fires one
        /// of these events.
        fn register_fd(&mut self, fd: RawFd, flags: FdFlags) -> Result<(), FdError>;
        /// Updates the set of events a given File Descriptor will fire.
        fn modify_fd(&mut self, fd: RawFd, flags: FdFlags) -> Result<(), FdError>;
        /// Removes a given File Descriptor from the host's event reactor.
        fn unregister_fd(&mut self, fd: RawFd) -> Result<(), FdError>;
    }

    impl<H: for<'a> Host<'a>> ExtensionImplementation<H> for HostPosixFd
    where
        for<'a> <H as Host<'a>>::MainThread: HostPosixFdImplementation,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: &'static Self = &Self(clap_host_posix_fd_support {
            register_fd: Some(register_fd::<H>),
            modify_fd: Some(modify_fd::<H>),
            unregister_fd: Some(unregister_fd::<H>),
        });
    }

    unsafe extern "C" fn register_fd<H: for<'a> Host<'a>>(
        host: *const clap_host,
        fd: i32,
        flags: clap_posix_fd_flags,
    ) -> bool
    where
        for<'a> <H as Host<'a>>::MainThread: HostPosixFdImplementation,
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

    unsafe extern "C" fn modify_fd<H: for<'a> Host<'a>>(
        host: *const clap_host,
        fd: i32,
        flags: clap_posix_fd_flags,
    ) -> bool
    where
        for<'a> <H as Host<'a>>::MainThread: HostPosixFdImplementation,
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
    unsafe extern "C" fn unregister_fd<H: for<'a> Host<'a>>(host: *const clap_host, fd: i32) -> bool
    where
        for<'a> <H as Host<'a>>::MainThread: HostPosixFdImplementation,
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
    use clack_common::extensions::ExtensionImplementation;
    use clack_plugin::host::HostMainThreadHandle;
    use clack_plugin::plugin::wrapper::PluginWrapper;
    use clack_plugin::plugin::Plugin;
    use clap_sys::plugin::clap_plugin;
    use std::os::unix::prelude::RawFd;

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
            let register_fd = self.0.register_fd.ok_or(FdError::Register((fd, flags)))?;

            let success = unsafe { register_fd(host.as_raw(), fd, flags.bits) };
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
            let modify_fd = self.0.modify_fd.ok_or(FdError::Modify((fd, flags)))?;

            let success = unsafe { modify_fd(host.as_raw(), fd, flags.bits) };
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
            let unregister_fd = self.0.unregister_fd.ok_or(FdError::Unregister(fd))?;

            let success = unsafe { unregister_fd(host.as_raw(), fd) };
            match success {
                true => Ok(()),
                false => Err(FdError::Unregister(fd)),
            }
        }
    }

    /// Implementation of the Plugin-side of the POSIX File Descriptors extension.
    pub trait PluginPosixFdImplementation {
        /// A callback that gets called for every event on each registered File Descriptor.
        ///
        /// Note this callback is "level-triggered". It means that for instance, a writable File
        /// Descriptor will continuously produce "on_fd()" events.
        ///
        /// Don't forget to use the `modify_fd` method to remove the write notification once you're
        /// done writing.
        fn on_fd(&mut self, fd: RawFd, flags: FdFlags);
    }

    impl<H: for<'a> Plugin<'a>> ExtensionImplementation<H> for PluginPosixFd
    where
        for<'a> <H as Plugin<'a>>::MainThread: PluginPosixFdImplementation,
    {
        #[doc(hidden)]
        const IMPLEMENTATION: &'static Self = &Self(clap_plugin_posix_fd_support {
            on_fd: Some(on_fd::<H>),
        });
    }

    unsafe extern "C" fn on_fd<H: for<'a> Plugin<'a>>(
        plugin: *const clap_plugin,
        fd: i32,
        flags: clap_posix_fd_flags,
    ) where
        for<'a> <H as Plugin<'a>>::MainThread: PluginPosixFdImplementation,
    {
        PluginWrapper::<H>::handle(plugin, |plugin| {
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
