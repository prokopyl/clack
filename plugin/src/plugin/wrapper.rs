use crate::plugin::error::PluginInternalError;
use crate::plugin::{wrapper, Plugin, PluginData, PluginInnerData};
use clap_audio_common::extensions::log::implementation::StdoutLogger;
use clap_audio_common::extensions::log::{Log, LogSeverity};
use clap_audio_common::host::HostHandle;
use clap_sys::plugin::clap_plugin;
use std::error::Error;
use std::panic::AssertUnwindSafe;

#[cfg(not(test))]
use std::panic::catch_unwind;

#[cfg(test)]
#[inline]
pub fn catch_unwind<F: FnOnce() -> R + std::panic::UnwindSafe, R>(f: F) -> std::thread::Result<R> {
    Ok(f())
}

unsafe fn get_logger<'a, P: Plugin<'a>>(
    plugin: *const clap_plugin,
) -> Option<(HostHandle<'a>, &'a Log)> {
    let host = plugin
        .as_ref()?
        .plugin_data
        .cast::<PluginData<'a, P>>()
        .as_ref()?
        .host;

    Some((host, host.extension()?))
}

unsafe fn handle<'a, P: Plugin<'a>, T, F, E>(
    plugin: *const clap_plugin,
    handler: F,
) -> Result<T, PluginInternalError<E>>
where
    F: FnOnce(&PluginInnerData<'a, P>) -> Result<T, E>,
    E: Error,
{
    let plugin = plugin.as_ref().ok_or(PluginInternalError::NulPluginDesc)?;
    let plugin = plugin
        .plugin_data
        .cast::<PluginData<'a, P>>()
        .as_ref()
        .ok_or(PluginInternalError::NulPluginData)?;

    let plugin = plugin
        .plugin_data
        .as_ref()
        .ok_or(PluginInternalError::UninitializedPlugin)?;

    // TODO: AssertUnwindSafe may not be good here
    Ok(catch_unwind(AssertUnwindSafe(|| handler(plugin)))
        .map_err(|_| PluginInternalError::Panic)??)
}

unsafe fn handle_mut<'a, P: Plugin<'a>, T, F, E>(
    plugin: *const clap_plugin,
    handler: F,
) -> Result<T, PluginInternalError<E>>
where
    F: FnOnce(&mut PluginInnerData<'a, P>) -> Result<T, E>,
    E: Error,
{
    let plugin = plugin.as_ref().ok_or(PluginInternalError::NulPluginDesc)?;
    let plugin = plugin
        .plugin_data
        .cast::<PluginData<'a, P>>()
        .as_mut()
        .ok_or(PluginInternalError::NulPluginData)?;

    let plugin = plugin
        .plugin_data
        .as_mut()
        .ok_or(PluginInternalError::UninitializedPlugin)?;

    // TODO: AssertUnwindSafe may not be good here
    Ok(catch_unwind(AssertUnwindSafe(|| handler(plugin)))
        .map_err(|_| PluginInternalError::Panic)??)
}

unsafe fn log_safe<'a, P: Plugin<'a>, E: Error>(
    plugin: *const clap_plugin,
    e: PluginInternalError<E>,
) {
    use clap_audio_common::extensions::log::implementation::HostLog;
    if let Some((host, logger)) = get_logger::<P>(plugin) {
        match logger.log_display(&host, e.severity(), &e) {
            Ok(()) => return,
            Err(e) => StdoutLogger.log(
                LogSeverity::Fatal,
                &format!("Failed to serialize error message for host: {}", e),
            ),
        };
    }

    StdoutLogger.log(e.severity(), &format!("{}", e));
}

/// # Safety
/// The plugin pointer must be valid
// TODO: cleanup this sometime
pub unsafe fn handle_plugin<'a, P: Plugin<'a>, F, E>(plugin: *const clap_plugin, handler: F) -> bool
where
    F: FnOnce(&PluginInnerData<'a, P>) -> Result<(), E>,
    E: Error,
{
    match wrapper::handle(plugin, handler) {
        Ok(()) => true,
        Err(e) => {
            wrapper::log_safe::<P, _>(plugin, e);

            false
        }
    }
}

/// # Safety
/// The plugin pointer must be valid
// TODO: cleanup this sometime
pub unsafe fn handle_plugin_mut<'a, P: Plugin<'a>, F, E>(
    plugin: *const clap_plugin,
    handler: F,
) -> bool
where
    F: FnOnce(&mut PluginInnerData<'a, P>) -> Result<(), E>,
    E: Error,
{
    match wrapper::handle_mut(plugin, handler) {
        Ok(()) => true,
        Err(e) => {
            wrapper::log_safe::<P, _>(plugin, e);

            false
        }
    }
}

/// # Safety
/// The plugin pointer must be valid
pub unsafe fn handle_plugin_returning<'a, P: Plugin<'a>, T, F, E>(
    plugin: *const clap_plugin,
    handler: F,
) -> Option<T>
where
    F: FnOnce(&PluginInnerData<'a, P>) -> Result<T, E>,
    E: Error,
{
    match handle(plugin, handler) {
        Ok(value) => Some(value),
        Err(e) => {
            log_safe::<P, _>(plugin, e);

            None
        }
    }
}
