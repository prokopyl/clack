use clack_common::extensions::{Extension, HostExtension};
use clap_sys::ext::log::{clap_host_log, clap_log_severity, CLAP_EXT_LOG};

mod error;
#[cfg(feature = "clack-host")]
pub mod implementation;
pub use error::LogError;

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum LogSeverity {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
    Fatal = 4,

    HostMisbehaving = 5,
    PluginMisbehaving = 6,
}

impl LogSeverity {
    pub(crate) fn from_raw(raw: clap_log_severity) -> Option<Self> {
        use clap_sys::ext::log::*;
        use LogSeverity::*;

        match raw {
            CLAP_LOG_DEBUG => Some(Debug),
            CLAP_LOG_INFO => Some(Info),
            CLAP_LOG_WARNING => Some(Warning),
            CLAP_LOG_ERROR => Some(Error),
            CLAP_LOG_FATAL => Some(Fatal),
            CLAP_LOG_HOST_MISBEHAVING => Some(HostMisbehaving),
            CLAP_LOG_PLUGIN_MISBEHAVING => Some(PluginMisbehaving),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn to_raw(self) -> clap_log_severity {
        self as _
    }
}

#[repr(C)]
pub struct Log(clap_host_log);

unsafe impl<'a> Extension<'a> for Log {
    const IDENTIFIER: *const u8 = CLAP_EXT_LOG as *const _;
    type ExtensionType = HostExtension;
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::host::HostHandle;
    use std::ffi::CStr;

    impl Log {
        #[inline]
        pub fn log(&self, host: &HostHandle, log_severity: LogSeverity, message: &CStr) {
            unsafe { (self.0.log)(host.as_raw(), log_severity.to_raw(), message.as_ptr()) }
        }
    }
}
