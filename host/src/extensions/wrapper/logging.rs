use crate::extensions::prelude::HostWrapperError;
use clap_sys::ext::log::{CLAP_EXT_LOG, clap_host_log, clap_log_severity};
use clap_sys::host::clap_host;
use std::os::raw::c_char;

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

/// # Safety
///
/// Plugin pointer must be non-dangling (but can be NULL).
/// It *must* point to a plugin instance created by Clack.
pub unsafe fn host_log(host: *const clap_host, e: &HostWrapperError) {
    if let Some(logger) = get_logger(host) {
        let cstr = e.format_cstr();
        logger(host, e.severity(), cstr.as_ptr());
    } else {
        eprintln!("[CLAP_HOST_ERROR] {e}");
    }
}
