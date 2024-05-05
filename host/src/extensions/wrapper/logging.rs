use crate::extensions::prelude::HostWrapperError;
use clap_sys::ext::log::{clap_host_log, clap_log_severity, CLAP_EXT_LOG};
use clap_sys::host::clap_host;
use std::os::raw::c_char;
use std::{error::Error, ffi::CString, fmt::Write};

pub type ClapLoggingFn =
    unsafe extern "C" fn(host: *const clap_host, severity: clap_log_severity, msg: *const c_char);

/// # Safety
///
/// Host pointer must be non-dangling (but can be NULL).
/// It *must* point to a valid clap_host instance if not NULL.
unsafe fn get_logger(host: *const clap_host) -> Option<ClapLoggingFn> {
    let host = host.as_ref()?;

    let log = host.get_extension?(host, CLAP_EXT_LOG.as_ptr()) as *mut clap_host_log;
    log.as_ref()?.log
}

fn log_display(error: &HostWrapperError) -> Result<CString, Box<dyn Error>> {
    let mut buf = String::new();
    write!(buf, "{}", error.msg())?;
    Ok(CString::new(buf)?)
}

/// # Safety
///
/// Plugin pointer must be non-dangling (but can be NULL).
/// It *must* point to a plugin instance created by Clack.
pub unsafe fn host_log(host: *const clap_host, e: &HostWrapperError) {
    if let Some(logger) = get_logger(host) {
        match log_display(e) {
            Ok(cstr) => {
                logger(host, e.severity(), cstr.as_ptr());
                return;
            }
            Err(e) => {
                eprintln!("[CLAP_HOST_ERROR] Failed to serialize error message for logging: {e}")
            }
        };
    }

    eprintln!("[CLAP_HOST_ERROR] {}", e.msg());
}
