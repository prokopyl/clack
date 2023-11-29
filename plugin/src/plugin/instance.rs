use crate::extensions::wrapper::PluginWrapper;
use crate::extensions::PluginExtensions;
use crate::host::{HostHandle, HostInfo};
use crate::plugin::descriptor::PluginDescriptorWrapper;
use crate::plugin::instance::WrapperState::*;
use crate::plugin::{
    AudioConfiguration, Plugin, PluginAudioProcessor, PluginError, PluginMainThread,
};
use crate::process::{Audio, Events, Process};
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use clap_sys::process::{clap_process, clap_process_status, CLAP_PROCESS_ERROR};
use core::ffi::c_void;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

pub(crate) trait InstanceInitializer<'a, P: Plugin>: 'a {
    fn init_shared(self: Box<Self>, handle: HostHandle<'a>) -> Result<P::Shared<'a>, PluginError>;
}

impl<'a, P: Plugin, F: 'a> InstanceInitializer<'a, P> for F
where
    F: FnOnce(HostHandle<'a>) -> Result<P::Shared<'a>, PluginError>,
{
    #[inline]
    fn init_shared(self: Box<Self>, handle: HostHandle<'a>) -> Result<P::Shared<'a>, PluginError> {
        self(handle)
    }
}

enum WrapperState<'a, P: Plugin> {
    Initialized(PluginWrapper<'a, P>),
    Uninitialized(Option<Box<dyn InstanceInitializer<'a, P>>>),
    InitializationFailed,
}

pub(crate) struct PluginBoxInner<'a, P: Plugin> {
    host: HostHandle<'a>,
    plugin_data: WrapperState<'a, P>,
}

impl<'a, P: Plugin> PluginBoxInner<'a, P> {
    #[inline]
    pub(crate) fn wrapper(&self) -> Option<&PluginWrapper<'a, P>> {
        match &self.plugin_data {
            Initialized(w) => Some(w),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn wrapper_mut(&mut self) -> Option<&mut PluginWrapper<'a, P>> {
        match &mut self.plugin_data {
            Initialized(w) => Some(w),
            _ => None,
        }
    }
}

impl<'a, P: Plugin> PluginBoxInner<'a, P> {
    fn get_plugin_desc(
        host: HostHandle<'a>,
        desc: &'a clap_plugin_descriptor,
        initializer: Option<Box<dyn InstanceInitializer<'a, P>>>,
    ) -> clap_plugin {
        clap_plugin {
            desc,
            plugin_data: Box::into_raw(Box::new(Self {
                host,
                plugin_data: Uninitialized(initializer),
            }))
            .cast(),
            init: Some(Self::init),
            destroy: Some(Self::destroy),
            activate: Some(Self::activate),
            deactivate: Some(Self::deactivate),
            reset: Some(Self::reset),
            start_processing: Some(Self::start_processing),
            stop_processing: Some(Self::stop_processing),
            process: Some(Self::process),
            get_extension: Some(Self::get_extension),
            on_main_thread: Some(Self::on_main_thread),
        }
    }

    #[inline]
    pub fn host(&self) -> &HostHandle<'a> {
        &self.host
    }

    unsafe extern "C" fn init(plugin: *const clap_plugin) -> bool {
        // TODO: null check this
        let data = &mut *((*plugin).plugin_data as *mut PluginBoxInner<'a, P>);
        let uninit_data = match &mut data.plugin_data {
            data @ Uninitialized(_) => core::mem::replace(data, InitializationFailed),
            Initialized(_) => {
                // TODO: use proper logging
                eprintln!("Plugin is already initialized");
                return true; // TODO: revert
            }
            InitializationFailed => {
                // TODO: use proper logging
                eprintln!("Plugin already failed to initialize");
                return true; // TODO: revert
            }
        };

        let Uninitialized(initializer) = uninit_data else {
            unreachable!()
        };

        let wrapper = match PluginWrapper::new(data.host.as_main_thread_unchecked(), initializer) {
            Ok(d) => d,
            Err(e) => {
                super::logging::plugin_log::<P>(plugin, &e.into());
                return false;
            }
        };

        data.plugin_data = Initialized(wrapper);

        true
    }

    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        let plugin = Box::from_raw(plugin as *mut clap_plugin);

        if !plugin.plugin_data.is_null() {
            let _ = Box::from_raw(plugin.plugin_data.cast::<PluginBoxInner<'a, P>>());
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
        PluginWrapper::<P>::handle_plugin_mut(plugin, |p| {
            p.audio_processor()?.as_mut().reset();
            Ok(())
        });
    }

    unsafe extern "C" fn start_processing(plugin: *const clap_plugin) -> bool {
        PluginWrapper::<P>::handle(plugin, |p| {
            Ok(p.audio_processor()?.as_mut().start_processing()?)
        })
        .is_some()
    }

    unsafe extern "C" fn stop_processing(plugin: *const clap_plugin) {
        PluginWrapper::<P>::handle(plugin, |p| {
            p.audio_processor()?.as_mut().stop_processing();
            Ok(())
        });
    }

    unsafe extern "C" fn process(
        plugin: *const clap_plugin,
        process: *const clap_process,
    ) -> clap_process_status {
        // SAFETY: process ptr is never accessed later, and is guaranteed to be valid and unique by the host
        PluginWrapper::<P>::handle(plugin, |p| {
            Ok(p.audio_processor()?.as_mut().process(
                Process::from_raw(&*process),
                Audio::from_raw(&*process),
                Events::from_raw(&*process),
            )?)
        })
        .map(|s| s as clap_process_status)
        .unwrap_or(CLAP_PROCESS_ERROR)
    }

    unsafe extern "C" fn get_extension(
        plugin: *const clap_plugin,
        identifier: *const std::os::raw::c_char,
    ) -> *const c_void {
        let identifier = CStr::from_ptr(identifier);
        let mut builder = PluginExtensions::new(identifier);

        PluginWrapper::<P>::handle(plugin, |p| {
            P::declare_extensions(&mut builder, p.shared());
            Ok(())
        });
        builder.found()
    }

    unsafe extern "C" fn on_main_thread(plugin: *const clap_plugin) {
        PluginWrapper::<P>::handle(plugin, |p| {
            p.main_thread().as_mut().on_main_thread();
            Ok(())
        });
    }
}

/// A wrapper around a [`Plugin`] instance.
///
/// This type is created with its [`new`](PluginInstance::new) method when the host wants to
/// instantiate a given plugin type, and is what needs to be returned by the
/// [`PluginFactory::instantiate_plugin`](crate::factory::plugin::PluginFactory::instantiate_plugin) method.
pub struct PluginInstance<'a> {
    inner: Box<clap_plugin>,
    lifetime: PhantomData<&'a clap_plugin_descriptor>,
}

impl<'a> PluginInstance<'a> {
    #[inline]
    pub(crate) fn into_owned_ptr(self) -> *mut clap_plugin {
        ManuallyDrop::new(self).inner.as_mut()
    }

    /// Instantiates a plugin of a given implementation `P`.
    ///
    /// Instantiated plugins also require an [`HostInfo`] instance given by the host, and a
    /// reference to the associated [`PluginDescriptorWrapper`].
    ///
    /// See the [`PluginFactory`](crate::factory::plugin::PluginFactory)'s trait documentation for
    /// an usage example.
    pub fn new<P: Plugin>(
        host_info: HostInfo<'a>,
        descriptor: &'a PluginDescriptorWrapper,
    ) -> PluginInstance<'a> {
        // SAFETY: we guarantee that no host_handle methods are called until init() is called
        let host = unsafe { host_info.to_handle() };
        Self {
            inner: Box::new(PluginBoxInner::<P>::get_plugin_desc(
                host,
                descriptor.as_raw(),
                None,
            )),
            lifetime: PhantomData,
        }
    }
    pub fn new_with<P: Plugin>(
        host_info: HostInfo<'a>,
        descriptor: &'a PluginDescriptorWrapper,
        initializer: impl FnOnce(HostHandle<'a>) -> Result<P::Shared<'a>, PluginError> + 'a,
    ) -> PluginInstance<'a> {
        // SAFETY: we guarantee that no host_handle methods are called until init() is called
        let host = unsafe { host_info.to_handle() };
        Self {
            inner: Box::new(PluginBoxInner::<P>::get_plugin_desc(
                host,
                descriptor.as_raw(),
                Some(Box::new(initializer) as Box<dyn InstanceInitializer<'a, P>>),
            )),
            lifetime: PhantomData,
        }
    }

    /// Returns a raw, C FFI-compatible reference to this plugin instance.
    #[inline]
    pub fn as_raw(&self) -> &clap_plugin {
        &self.inner
    }
}

// In case the instance is dropped by a faulty plugin factory implementation.
impl<'a> Drop for PluginInstance<'a> {
    #[inline]
    fn drop(&mut self) {
        if let Some(destroy) = self.inner.destroy {
            unsafe { destroy(self.inner.as_ref()) }
        }
    }
}
