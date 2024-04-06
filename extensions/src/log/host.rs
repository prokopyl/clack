use super::{HostLog, LogSeverity};
use clack_host::extensions::prelude::*;
use clap_sys::ext::log::{clap_host_log, clap_log_severity};
use std::borrow::Cow::Owned;
use std::ffi::CStr;
use std::os::raw::c_char;

pub trait HostLogImpl {
    fn log(&self, severity: LogSeverity, message: &str);
}

impl<H: HostHandlers> ExtensionImplementation<H> for HostLog
where
    for<'a> <H as HostHandlers>::Shared<'a>: HostLogImpl,
{
    const IMPLEMENTATION: RawExtensionImplementation =
        RawExtensionImplementation::new(&clap_host_log {
            log: Some(log::<H>),
        });
}

#[allow(clippy::missing_safety_doc)]
unsafe extern "C" fn log<H: HostHandlers>(
    host: *const clap_host,
    severity: clap_log_severity,
    msg: *const c_char,
) where
    for<'a> <H as HostHandlers>::Shared<'a>: HostLogImpl,
{
    let msg = CStr::from_ptr(msg).to_string_lossy();
    let res = HostWrapper::<H>::handle(host, |host| {
        let host = host.shared();
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
                &format!("Plugin logged with unknown log level: {severity}"),
            );
        }

        Ok(())
    });

    if res.is_none() {
        eprintln!("[ERROR] Log handler failed when writing message: {msg}")
    }
}
