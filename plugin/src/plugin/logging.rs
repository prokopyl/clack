use crate::plugin::wrapper::PluginWrapperError;
use crate::plugin::{Plugin, PluginInstanceImpl};
use clap_sys::ext::log::{clap_host_log, CLAP_EXT_LOG};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use std::{error::Error, ffi::CString, fmt::Display, fmt::Write};

unsafe fn get_logger<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
) -> Option<(&'a clap_host, &'a clap_host_log)> {
    let host = plugin
        .as_ref()?
        .plugin_data
        .cast::<PluginInstanceImpl<'a, P>>()
        .as_ref()?
        .host()
        .as_raw();

    let log = (host.get_extension)(host, CLAP_EXT_LOG as *const _) as *mut clap_host_log;
    Some((host, log.as_ref()?))
}

fn log_display<D: Display>(message: &D) -> Result<CString, Box<dyn Error>> {
    let mut buf = String::new();
    write!(buf, "{}", message)?;
    Ok(CString::new(buf)?)
}

pub unsafe fn plugin_log<'a, P: Plugin<'a>>(plugin: *const clap_plugin, e: &PluginWrapperError) {
    if let Some((host, logger)) = get_logger::<P>(plugin) {
        match log_display(e) {
            Ok(cstr) => {
                (logger.log)(host, e.severity(), cstr.as_ptr());
                return;
            }
            Err(e) => eprintln!(
                "[CLAP_PLUGIN_ERROR] Failed to serialize error message for host: {}",
                e
            ),
        };
    }

    eprintln!("[CLAP_PLUGIN_ERROR] {}", e);
}
