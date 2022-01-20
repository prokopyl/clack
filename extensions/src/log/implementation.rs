use super::{Log, LogSeverity};
use clack_common::extensions::ExtensionImplementation;
use clap_sys::ext::log::{clap_host_log, clap_log_severity};
use clap_sys::host::clap_host;
use std::borrow::Cow::Owned;
use std::ffi::CStr;
use std::os::raw::c_char;

pub trait HostLog {
    fn log(&self, severity: LogSeverity, message: &str);
}

unsafe impl<H: HostLog> ExtensionImplementation<H> for Log {
    type Interface = clap_host_log;
    const INTERFACE: &'static Self::Interface = &clap_host_log { log: log::<H> };
}

unsafe extern "C" fn log<H: HostLog>(
    host: *const clap_host,
    severity: clap_log_severity,
    msg: *const c_char,
) {
    let host = &*((*host).host_data as *const H);
    let msg = CStr::from_ptr(msg);
    let msg = msg.to_string_lossy();
    let log_severity = LogSeverity::from_raw(severity);

    H::log(
        host,
        log_severity.unwrap_or(LogSeverity::Warning),
        msg.as_ref(),
    );

    if let Owned(_) = msg {
        H::log(
            host,
            LogSeverity::PluginMisbehaving,
            "Plugin logged invalid UTF-8 data. Some characters may be invalid.",
        );
    }

    if log_severity.is_none() {
        H::log(
            host,
            LogSeverity::PluginMisbehaving,
            &format!("Plugin logged with unknown log level: {}", severity),
        );
    }
}
