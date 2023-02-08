use crate::plugin::wrapper::PluginWrapperError;
use crate::plugin::{Plugin, PluginInstanceImpl};
use clap_sys::ext::log::{clap_host_log, clap_log_severity, CLAP_EXT_LOG};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use std::os::raw::c_char;
use std::{error::Error, ffi::CString, fmt::Display, fmt::Write};

pub type ClapLoggingFn =
    unsafe extern "C" fn(host: *const clap_host, severity: clap_log_severity, msg: *const c_char);

unsafe fn get_logger<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
) -> Option<(*const clap_host, ClapLoggingFn)> {
    let host = plugin
        .as_ref()?
        .plugin_data
        .cast::<PluginInstanceImpl<'a, P>>()
        .as_ref()?
        .host()
        .as_raw();

    let log = ((*host).get_extension?)(host, CLAP_EXT_LOG.as_ptr()) as *mut clap_host_log;
    Some((host, log.as_ref()?.log?))
}

fn log_display<D: Display>(message: &D) -> Result<CString, Box<dyn Error>> {
    let mut buf = String::new();
    write!(buf, "{message}")?;
    Ok(CString::new(buf)?)
}

pub unsafe fn plugin_log<'a, P: Plugin<'a>>(plugin: *const clap_plugin, e: &PluginWrapperError) {
    if let Some((host, logger)) = get_logger::<P>(plugin) {
        match log_display(e) {
            Ok(cstr) => {
                logger(host, e.severity(), cstr.as_ptr());
                return;
            }
            Err(e) => {
                eprintln!("[CLAP_PLUGIN_ERROR] Failed to serialize error message for host: {e}")
            }
        };
    }

    eprintln!("[CLAP_PLUGIN_ERROR] {e}");
}
