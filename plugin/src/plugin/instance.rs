use crate::extensions::PluginExtensions;
use crate::host::{HostHandle, HostInfo};
use crate::plugin::wrapper::PluginWrapper;
use crate::plugin::{wrapper, AudioConfiguration, Plugin, PluginMainThread};
use crate::process::Process;
use clap_sys::plugin::clap_plugin;
use clap_sys::process::{clap_process, clap_process_status, CLAP_PROCESS_ERROR};
use core::ffi::c_void;
use std::ffi::CStr;
use std::marker::PhantomData;

pub(crate) struct PluginInstanceImpl<'a, P: Plugin<'a>> {
    host: HostHandle<'a>,
    pub(crate) plugin_data: Option<PluginWrapper<'a, P>>,
}

impl<'a, P: Plugin<'a>> PluginInstanceImpl<'a, P> {
    fn get_plugin_desc(self) -> clap_plugin {
        clap_plugin {
            desc: &P::DESCRIPTOR.0,
            plugin_data: Box::into_raw(Box::new(self)).cast(),
            init: Self::init,
            destroy: Self::destroy,
            activate: Self::activate,
            deactivate: Self::deactivate,
            reset: Self::reset,
            start_processing: Self::start_processing,
            stop_processing: Self::stop_processing,
            process: Self::process,
            get_extension: Self::get_extension,
            on_main_thread: Self::on_main_thread,
        }
    }

    #[inline]
    pub fn host(&self) -> &HostHandle<'a> {
        &self.host
    }

    unsafe extern "C" fn init(plugin: *const clap_plugin) -> bool {
        // TODO: null check this
        let data = &mut *((*plugin).plugin_data as *mut PluginInstanceImpl<'a, P>);
        if data.plugin_data.is_some() {
            eprintln!("Plugin is already initialized");
            return true; // TODO: revert
        }

        data.plugin_data = Some(match PluginWrapper::new(data.host.to_main_thread()) {
            Ok(d) => d,
            Err(e) => {
                super::logging::plugin_log::<P>(plugin, &e.into());
                return false;
            }
        });

        true
    }

    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        let plugin = Box::from_raw(plugin as *mut clap_plugin);

        if !plugin.plugin_data.is_null() {
            Box::from_raw(plugin.plugin_data.cast::<PluginInstanceImpl<'a, P>>());
        }
    }

    unsafe extern "C" fn activate(
        plugin: *const clap_plugin,
        sample_rate: f64,
        min_sample_count: u32,
        max_sample_count: u32,
    ) -> bool {
        PluginWrapper::<P>::handle_plugin_mut(plugin, |p| {
            let config = AudioConfiguration {
                sample_rate,
                min_sample_count,
                max_sample_count,
            };

            p.activate(config)
        })
        .is_some()
    }

    unsafe extern "C" fn deactivate(plugin: *const clap_plugin) {
        PluginWrapper::<P>::handle_plugin_mut(plugin, |p| p.deactivate());
    }

    unsafe extern "C" fn reset(plugin: *const clap_plugin) {
        PluginWrapper::<P>::handle_plugin_mut(plugin, |p| p.reset());
    }

    unsafe extern "C" fn start_processing(plugin: *const clap_plugin) -> bool {
        PluginWrapper::<P>::handle(plugin, |p| {
            Ok(P::start_processing(p.audio_processor()?.as_mut())?)
        })
        .is_some()
    }

    unsafe extern "C" fn stop_processing(plugin: *const clap_plugin) {
        PluginWrapper::<P>::handle(plugin, |p| {
            P::stop_processing(p.audio_processor()?.as_mut());
            Ok(())
        });
    }

    unsafe extern "C" fn process(
        plugin: *const clap_plugin,
        process: *const clap_process,
    ) -> clap_process_status {
        // SAFETY: process ptr is never accessed later, and is guaranteed to be valid and unique by the host
        let (process, audio, events) = Process::from_raw(process);
        PluginWrapper::<P>::handle(plugin, |p| {
            Ok(P::process(
                p.audio_processor()?.as_mut(),
                process,
                audio,
                events,
            )?)
        })
        .map(|s| s as clap_process_status)
        .unwrap_or(CLAP_PROCESS_ERROR as i32)
    }

    unsafe extern "C" fn get_extension(
        plugin: *const clap_plugin,
        identifier: *const std::os::raw::c_char,
    ) -> *const c_void {
        let identifier = CStr::from_ptr(identifier);
        let mut builder = PluginExtensions::new(identifier);

        wrapper::PluginWrapper::<P>::handle(plugin, |p| {
            P::declare_extensions(&mut builder, p.shared());
            Ok(())
        });
        builder.found()
    }

    unsafe extern "C" fn on_main_thread(plugin: *const clap_plugin) {
        wrapper::PluginWrapper::<P>::handle(plugin, |p| {
            p.main_thread().as_mut().on_main_thread();
            Ok(())
        });
    }
}

pub struct PluginInstance<'a> {
    inner: Box<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin>,
}

impl<'a> PluginInstance<'a> {
    #[inline]
    pub(crate) fn into_owned_ptr(self) -> *mut clap_plugin {
        Box::into_raw(self.inner)
    }

    pub fn new<P: Plugin<'a>>(host_info: HostInfo<'a>) -> PluginInstance<'a> {
        // SAFETY: we guarantee that no host_handle methods are called until init() is called
        let host = unsafe { host_info.to_handle() };
        let data = PluginInstanceImpl::<'a, P> {
            host,
            plugin_data: None,
        };
        Self {
            inner: Box::new(PluginInstanceImpl::<P>::get_plugin_desc(data)),
            lifetime: PhantomData,
        }
    }
}
