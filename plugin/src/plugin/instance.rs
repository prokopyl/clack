use crate::extension::ExtensionDeclarations;
use crate::plugin::error::PluginInternalError;
use crate::plugin::{wrapper, Plugin, PluginError, PluginMainThread, PluginShared, SampleConfig};
use crate::process::Process;
use clap_audio_common::host::{HostHandle, HostInfo};
use clap_sys::plugin::clap_plugin;
use clap_sys::process::{clap_process, clap_process_status, CLAP_PROCESS_ERROR};
use core::ffi::c_void;
use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

// TODO: bikeshed
pub struct PluginInnerData<'a, P: Plugin<'a>> {
    shared: P::Shared,
    main_thread: UnsafeCell<P::MainThread>,
    audio_processor: Option<UnsafeCell<P>>,
}

impl<'a, P: Plugin<'a>> PluginInnerData<'a, P> {
    pub fn new(host: HostHandle<'a>) -> Result<Self, PluginError> {
        let shared = P::Shared::new(host)?;
        let main_thread = UnsafeCell::new(P::MainThread::new(host, &shared)?);

        Ok(Self {
            shared,
            main_thread,
            audio_processor: None,
        })
    }

    #[inline]
    pub fn shared(&self) -> &P::Shared {
        &self.shared
    }

    /// # Safety
    /// Caller must ensure this method is only called on main thread and has exclusivity
    pub unsafe fn activate(
        &mut self, // TODO: Pin this
        host: HostHandle<'a>,
        sample_config: SampleConfig,
    ) -> Result<(), PluginInternalError> {
        if self.audio_processor.is_some() {
            return Err(PluginInternalError::ActivatedPlugin);
        }

        // SAFETY: self should not move, pointer is valid for the lifetime of P
        let shared = &*(&self.shared as *const _);

        let processor = P::new(host, self.main_thread.get_mut(), shared, sample_config)?;
        self.audio_processor = Some(UnsafeCell::new(processor));

        Ok(())
    }

    /// # Safety
    /// Caller must ensure this method is only called on main thread, and has exclusivity on it
    pub unsafe fn deactivate(&mut self) -> Result<(), PluginInternalError> {
        if self.audio_processor.take().is_none() {
            return Err(PluginInternalError::DeactivatedPlugin);
        }

        Ok(())
    }

    /// # Safety
    /// Caller must ensure this method is only called on main thread, and has exclusivity on it
    #[inline]
    pub unsafe fn main_thread(&self) -> NonNull<P::MainThread> {
        // SAFETY: pointer has been created from reference
        NonNull::new_unchecked(self.main_thread.get())
    }

    /// # Safety
    /// Caller must ensure this method is only called on an audio thread, and has exclusivity on it
    #[inline]
    pub unsafe fn audio_processor(&self) -> Result<NonNull<P>, PluginInternalError> {
        self.audio_processor
            .as_ref()
            // SAFETY: pointer has been created from reference
            .map(|p| NonNull::new_unchecked(p.get()))
            .ok_or(PluginInternalError::DeactivatedPlugin)
    }
}

unsafe impl<'a, P: Plugin<'a>> Send for PluginInnerData<'a, P> {}
unsafe impl<'a, P: Plugin<'a>> Sync for PluginInnerData<'a, P> {}

pub(crate) struct PluginData<'a, P: Plugin<'a>> {
    pub(crate) host: HostHandle<'a>,
    pub(crate) plugin_data: Option<PluginInnerData<'a, P>>,
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

    fn get_plugin_desc<P: Plugin<'a>>(data: PluginData<'a, P>) -> clap_plugin {
        clap_plugin {
            desc: &P::DESCRIPTOR.0,
            plugin_data: Box::into_raw(Box::new(data)).cast(),
            init: Self::init::<P>,
            destroy: Self::destroy::<P>,
            activate: Self::activate::<P>,
            deactivate: Self::deactivate::<P>,
            start_processing: Self::start_processing::<P>,
            stop_processing: Self::stop_processing::<P>,
            process: Self::process::<P>,
            get_extension: Self::get_extension::<P>,
            on_main_thread: Self::on_main_thread::<P>,
        }
    }

    pub fn new<P: Plugin<'a>>(host_info: HostInfo<'a>) -> PluginInstance<'a> {
        // SAFETY: we guarantee that no host_handle methods are called until init() is called
        let host = unsafe { host_info.to_handle() };
        let data = PluginData::<'a, P> {
            host,
            plugin_data: None,
        };
        Self {
            inner: Box::new(Self::get_plugin_desc(data)),
            lifetime: PhantomData,
        }
    }

    unsafe extern "C" fn init<P: Plugin<'a>>(plugin: *const clap_plugin) -> bool {
        // TODO: null check this
        let data = &mut *((*plugin).plugin_data as *mut PluginData<'a, P>);
        if data.plugin_data.is_some() {
            eprintln!("Plugin is already initialized");
            return false;
        }

        data.plugin_data = Some(match PluginInnerData::new(data.host) {
            Ok(d) => d,
            Err(_) => {
                return false;
            } // TODO: properly display/log error
        });

        true
    }

    unsafe extern "C" fn destroy<P: Plugin<'a>>(plugin: *const clap_plugin) {
        let plugin = Box::from_raw(plugin as *mut clap_plugin);

        if !plugin.plugin_data.is_null() {
            Box::from_raw(plugin.plugin_data.cast::<PluginData<'a, P>>());
        }
    }

    unsafe extern "C" fn activate<P: Plugin<'a>>(
        plugin: *const clap_plugin,
        sample_rate: f64,
        min_sample_count: u32,
        max_sample_count: u32,
    ) -> bool {
        wrapper::handle_plugin_mut::<P, _, _>(plugin, |p| {
            let config = SampleConfig {
                sample_rate,
                min_sample_count,
                max_sample_count,
            };
            let host = (*((*plugin).plugin_data as *mut PluginData<'a, P>)).host;
            p.activate(host, config)
        })
    }

    unsafe extern "C" fn deactivate<P: Plugin<'a>>(plugin: *const clap_plugin) {
        wrapper::handle_plugin_mut::<P, _, PluginInternalError>(plugin, |p| p.deactivate());
    }

    unsafe extern "C" fn start_processing<P: Plugin<'a>>(plugin: *const clap_plugin) -> bool {
        wrapper::handle_plugin::<P, _, PluginInternalError>(plugin, |p| {
            Ok(P::start_processing(p.audio_processor()?.as_mut())?)
        })
    }

    unsafe extern "C" fn stop_processing<P: Plugin<'a>>(plugin: *const clap_plugin) {
        wrapper::handle_plugin::<P, _, PluginInternalError>(plugin, |p| {
            P::stop_processing(p.audio_processor()?.as_mut());
            Ok(())
        });
    }

    unsafe extern "C" fn process<P: Plugin<'a>>(
        plugin: *const clap_plugin,
        process: *const clap_process,
    ) -> clap_process_status {
        // SAFETY: process ptr is never accessed later, and is guaranteed to be valid and unique by the host
        let (process, audio, events) = Process::from_raw(process);
        wrapper::handle_plugin_returning::<P, _, _, PluginInternalError>(plugin, |p| {
            Ok(P::process(
                p.audio_processor()?.as_mut(),
                process,
                audio,
                events,
            )?)
        })
        .map(|s| s as clap_process_status)
        .unwrap_or(CLAP_PROCESS_ERROR)
    }

    unsafe extern "C" fn get_extension<P: Plugin<'a>>(
        plugin: *const clap_plugin,
        identifier: *const std::os::raw::c_char,
    ) -> *const c_void {
        let identifier = CStr::from_ptr(identifier);
        let mut builder = ExtensionDeclarations::new(identifier);

        wrapper::handle_plugin::<P, _, PluginError>(plugin, |p| {
            P::declare_extensions(&mut builder, p.shared());
            Ok(())
        });
        builder.found()
    }

    unsafe extern "C" fn on_main_thread<P: Plugin<'a>>(plugin: *const clap_plugin) {
        wrapper::handle_plugin::<P, _, PluginError>(plugin, |p| {
            p.main_thread().as_mut().on_main_thread();
            Ok(())
        });
    }
}
