use clack_common::extensions::{Extension, HostExtensionSide, RawExtension};
use clap_sys::ext::log::{clap_host_log, clap_log_severity, CLAP_EXT_LOG};
use std::ffi::CStr;
use std::fmt::{Display, Formatter};

mod error;
pub use error::LogError;

#[cfg(feature = "clack-host")]
mod host;

#[cfg(feature = "clack-host")]
pub use host::*;

#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
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

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct HostLog(RawExtension<HostExtensionSide, clap_host_log>);

// SAFETY: This type is repr(C) and ABI-compatible with the matching extension type.
unsafe impl Extension for HostLog {
    const IDENTIFIER: &'static CStr = CLAP_EXT_LOG;
    type ExtensionSide = HostExtensionSide;

    #[inline]
    unsafe fn from_raw(raw: RawExtension<Self::ExtensionSide>) -> Self {
        Self(raw.cast())
    }
}

#[cfg(feature = "clack-plugin")]
mod plugin {
    use super::*;
    use clack_plugin::host::HostSharedHandle;

    impl HostLog {
        #[inline]
        pub fn log(&self, host: &HostSharedHandle, log_severity: LogSeverity, message: &CStr) {
            if let Some(log) = host.use_extension(&self.0).log {
                // SAFETY: This type ensures the function pointer is valid.
                unsafe { log(host.as_raw(), log_severity.to_raw(), message.as_ptr()) }
            }
        }
    }
}
