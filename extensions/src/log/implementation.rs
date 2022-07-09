use super::{Log, LogSeverity};
use clack_common::extensions::ExtensionImplementation;
use clack_host::host::Host;
use clack_host::wrapper::HostWrapper;
use clap_sys::ext::log::{clap_host_log, clap_log_severity};
use clap_sys::host::clap_host;
use std::borrow::Cow::Owned;
use std::ffi::CStr;
use std::os::raw::c_char;

pub trait HostLog {
    fn log(&self, severity: LogSeverity, message: &str);
}

impl<H: for<'a> Host<'a>> ExtensionImplementation<H> for Log
where
    for<'a> <H as Host<'a>>::Shared: HostLog,
{
    const IMPLEMENTATION: &'static Self = &Log(clap_host_log {
        log: Some(log::<H>),
    });
}

unsafe extern "C" fn log<H: for<'a> Host<'a>>(
    host: *const clap_host,
    severity: clap_log_severity,
    msg: *const c_char,
) where
    for<'a> <H as Host<'a>>::Shared: HostLog,
{
    let _res = HostWrapper::<H>::handle(host, |host| {
        let host = host.shared();
        let msg = CStr::from_ptr(msg).to_string_lossy();
        let log_severity = LogSeverity::from_raw(severity);

        host.log(log_severity.unwrap_or(LogSeverity::Warning), msg.as_ref());

        if let Owned(_) = msg {
            host.log(
                LogSeverity::PluginMisbehaving,
                "Plugin logged invalid UTF-8 data. Some characters may be invalid.",
            );
        }

        if log_severity.is_none() {
            host.log(
                LogSeverity::PluginMisbehaving,
                &format!("Plugin logged with unknown log level: {}", severity),
            );
        }

        Ok(())
    });

    // TODO: perhaps write straight into STDERR if log error handler failed/panicked
}
