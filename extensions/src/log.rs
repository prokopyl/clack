use clack_common::extensions::{Extension, HostExtensionType};
use clap_sys::ext::log::{clap_host_log, clap_log_severity, CLAP_EXT_LOG};
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

mod error;
pub use error::LogError;

#[cfg(feature = "clack-host")]
mod implementation;

#[cfg(feature = "clack-host")]
pub use implementation::*;

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
    pub fn from_raw(raw: clap_log_severity) -> Option<Self> {
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
    pub fn to_raw(self) -> clap_log_severity {
        self as _
    }
}

impl Display for LogSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let displayed = match self {
            LogSeverity::Debug => "DEBUG",
            LogSeverity::Info => "INFO",
            LogSeverity::Warning => "WARN",
            LogSeverity::Error => "ERROR",
            LogSeverity::Fatal => "FATAL",
            LogSeverity::HostMisbehaving => "HOST_MISBEHAVING",
            LogSeverity::PluginMisbehaving => "PLUGIN_MISBEHAVING",
        };

        f.write_str(displayed)
    }
}

#[repr(C)]
pub struct HostLog(clap_host_log);

// SAFETY: The API of this extension makes it so that the Send/Sync requirements are enforced onto
// the input handles, not on the descriptor itself.
unsafe impl Send for HostLog {}
unsafe impl Sync for HostLog {}

unsafe impl Extension for HostLog {
    const IDENTIFIER: &'static CStr = CLAP_EXT_LOG;
    type ExtensionType = HostExtensionType;
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::host::HostHandle;
    use std::ffi::CStr;

    impl HostLog {
        #[inline]
        pub fn log(&self, host: &HostHandle, log_severity: LogSeverity, message: &CStr) {
            if let Some(log) = self.0.log {
                unsafe { log(host.as_raw(), log_severity.to_raw(), message.as_ptr()) }
            }
        }
    }
}
