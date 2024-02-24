use crate::extensions::wrapper::{PluginWrapper, PluginWrapperError};
use crate::extensions::PluginExtensions;
use crate::host::{HostHandle, HostInfo, HostMainThreadHandle};
use crate::plugin::instance::WrapperState::*;
use crate::plugin::{
    AudioConfiguration, Plugin, PluginAudioProcessor, PluginError, PluginMainThread,
};
use crate::prelude::PluginDescriptor;
use crate::process::{Audio, Events, Process};
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use clap_sys::process::{clap_process, clap_process_status, CLAP_PROCESS_ERROR};
use core::ffi::c_void;
use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

pub(crate) trait PluginInitializer<'a, P: Plugin>: 'a {
    fn init(
        self: Box<Self>,
        host: HostMainThreadHandle<'a>,
    ) -> Result<PluginWrapper<P>, PluginError>;
}

impl<'a, P: Plugin, FS: 'a, FM: 'a> PluginInitializer<'a, P> for (FS, FM)
where
    FS: FnOnce(HostHandle<'a>) -> Result<P::Shared<'a>, PluginError>,
    FM: FnOnce(
        HostMainThreadHandle<'a>,
        &'a P::Shared<'a>,
    ) -> Result<P::MainThread<'a>, PluginError>,
{
    fn init(
        self: Box<Self>,
        host: HostMainThreadHandle<'a>,
    ) -> Result<PluginWrapper<P>, PluginError> {
        let (shared_initializer, main_thread_initializer) = *self;
        let shared = Box::pin(shared_initializer(host.shared())?);

        // SAFETY: this lives long enough
        let shared_ref = unsafe { &*(shared.as_ref().get_ref() as *const _) };
        let main_thread = UnsafeCell::new(main_thread_initializer(host, shared_ref)?);

        Ok(unsafe { PluginWrapper::new(host.shared(), shared, main_thread) })
    }
}

impl<'a, P: Plugin, F: 'a, FM: 'a> PluginInitializer<'a, P> for F
where
    F: FnOnce(HostMainThreadHandle<'a>) -> Result<(P::Shared<'a>, FM), PluginError>,
    FM: FnOnce(&'a P::Shared<'a>) -> Result<P::MainThread<'a>, PluginError>,
{
    fn init(
        self: Box<Self>,
        host: HostMainThreadHandle<'a>,
    ) -> Result<PluginWrapper<P>, PluginError> {
        let (shared, main_thread_initializer) = self(host)?;
        let shared = Box::pin(shared);

        // SAFETY: this lives long enough
        let shared_ref = unsafe { &*(shared.as_ref().get_ref() as *const _) };
        let main_thread = UnsafeCell::new(main_thread_initializer(shared_ref)?);

        Ok(unsafe { PluginWrapper::new(host.shared(), shared, main_thread) })
    }
}

pub(crate) enum WrapperState<'a, P: Plugin> {
    Initialized(PluginWrapper<'a, P>),
    Uninitialized(Box<dyn PluginInitializer<'a, P>>),
    InitializationFailed,
}

pub(crate) struct PluginBoxInner<'a, P: Plugin> {
    host: HostHandle<'a>,
    plugin_data: WrapperState<'a, P>,
}

impl<'a, P: Plugin> PluginBoxInner<'a, P> {
    #[inline]
    pub(crate) fn wrapper(&self) -> Result<&PluginWrapper<'a, P>, PluginWrapperError> {
        match &self.plugin_data {
            Initialized(w) => Ok(w),
            InitializationFailed => Err(PluginWrapperError::InitializationAlreadyFailed),
            Uninitialized(_) => Err(PluginWrapperError::UninitializedPlugin),
        }
    }

    #[inline]
    pub(crate) fn wrapper_mut(&mut self) -> Result<&mut PluginWrapper<'a, P>, PluginWrapperError> {
        match &mut self.plugin_data {
            Initialized(w) => Ok(w),
            InitializationFailed => Err(PluginWrapperError::InitializationAlreadyFailed),
            Uninitialized(_) => Err(PluginWrapperError::UninitializedPlugin),
        }
    }
}

impl<'a, P: Plugin> PluginBoxInner<'a, P> {
    fn get_plugin_desc(
        host: HostHandle<'a>,
        desc: &'a clap_plugin_descriptor,
        initializer: Box<dyn PluginInitializer<'a, P>>,
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
        PluginWrapper::<P>::handle_plugin_data(plugin, |mut data| {
            let data = data.as_mut();

            let uninit_data = match &mut data.plugin_data {
                data @ Uninitialized(_) => Ok(core::mem::replace(data, InitializationFailed)),
                Initialized(_) => Err(PluginWrapperError::AlreadyInitialized),
                InitializationFailed => Err(PluginWrapperError::InitializationAlreadyFailed),
            }?;

            let Uninitialized(initializer) = uninit_data else {
                unreachable!()
            };

            let wrapper = initializer.init(data.host.as_main_thread_unchecked())?;

            data.plugin_data = Initialized(wrapper);
            Ok(())
        })
        .is_some()
    }

    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        PluginWrapper::<P>::handle_plugin_data(plugin, |data| {
            let _ = Box::from_raw(data.as_ptr());
            Ok(())
        });

        if let Some(plugin) = plugin.cast_mut().as_mut() {
            // Try our best to guard against double-free
            if plugin.plugin_data.is_null() {
                return;
            }

            plugin.plugin_data = core::ptr::null_mut();

            let _ = Box::from_raw(plugin as *mut clap_plugin);
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
/// [`PluginFactory::instantiate_plugin`](crate::factory::plugin::PluginFactory::create_plugin) method.
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
    /// reference to the associated [`PluginDescriptor`].
    ///
    /// See the [`PluginFactory`](crate::factory::plugin::PluginFactory)'s trait documentation for
    /// a usage example.
    pub fn new<P: Plugin>(
        host_info: HostInfo<'a>,
        descriptor: &'a PluginDescriptor,
        shared_initializer: impl FnOnce(HostHandle<'a>) -> Result<P::Shared<'a>, PluginError> + 'a,
        main_thread_initializer: impl FnOnce(
                HostMainThreadHandle<'a>,
                &'a P::Shared<'a>,
            ) -> Result<P::MainThread<'a>, PluginError>
            + 'a,
    ) -> PluginInstance<'a> {
        // SAFETY: we guarantee that no host_handle methods are called until init() is called
        let host = unsafe { host_info.to_handle() };
        Self {
            inner: Box::new(PluginBoxInner::<P>::get_plugin_desc(
                host,
                descriptor.as_raw(),
                Box::new((shared_initializer, main_thread_initializer))
                    as Box<dyn PluginInitializer<'a, P>>,
            )),
            lifetime: PhantomData,
        }
    }

    /// Instantiates a plugin of a given implementation `P` with a custom initializer.
    ///
    /// This initializer allows to perform custom work on the main thread at initialization time
    /// before the plugin is split into its shared and main thread components.
    ///
    /// Instantiated plugins also require an [`HostInfo`] instance given by the host, and a
    /// reference to the associated [`PluginDescriptor`].
    ///
    /// # Example
    ///
    /// This example shows how to use this API to create a [`PluginShared`] struct and a
    /// [`PluginMainThread`] struct that each use one end of a [`channel`].
    ///
    /// ```
    /// use std::sync::mpsc::{channel, Receiver, Sender};
    ///
    /// use clack_plugin::prelude::*;
    /// use clack_plugin::entry::prelude::*;
    ///
    /// struct MyPlugin;
    ///
    /// impl Plugin for MyPlugin {
    ///     type AudioProcessor<'a> = ();
    ///     type Shared<'a> = MyPluginShared<'a>;
    ///     type MainThread<'a> = MyPluginMainThread<'a>;
    /// }
    ///
    /// struct MyPluginShared<'a>(HostHandle<'a>, Sender<u32>);
    /// struct MyPluginMainThread<'a>(HostMainThreadHandle<'a>, &'a MyPluginShared<'a>, Receiver<u32>);
    ///
    /// impl<'a> PluginShared<'a> for MyPluginShared<'a> {}
    /// impl<'a> PluginMainThread<'a, MyPluginShared<'a>> for MyPluginMainThread<'a> {}
    ///
    /// fn create_plugin<'a>(host_info: HostInfo<'a>) -> Result<PluginInstance<'a>, PluginError> {
    ///     let plugin_descriptor: &'a PluginDescriptor = /* ... */
    /// #    unreachable!();
    ///     Ok(PluginInstance::new_with_initializer::<MyPlugin, _>(host_info, plugin_descriptor, |host| {
    ///         let (tx, rx) = channel();
    ///
    ///         Ok((
    ///             MyPluginShared(host.shared(), tx),
    ///             move |shared| Ok(MyPluginMainThread(host, shared, rx))
    ///         ))
    ///     }))
    /// }
    ///
    /// ```
    /// [`PluginShared`]: crate::plugin::PluginShared
    /// [`channel`]: std::sync::mpsc::channel
    pub fn new_with_initializer<P: Plugin, FM: 'a>(
        host_info: HostInfo<'a>,
        descriptor: &'a PluginDescriptor,
        initializer: impl FnOnce(HostMainThreadHandle<'a>) -> Result<(P::Shared<'a>, FM), PluginError>
            + 'a,
    ) -> PluginInstance<'a>
    where
        FM: FnOnce(&'a P::Shared<'a>) -> Result<P::MainThread<'a>, PluginError>,
    {
        // SAFETY: we guarantee that no host_handle methods are called until init() is called
        let host = unsafe { host_info.to_handle() };
        Self {
            inner: Box::new(PluginBoxInner::<P>::get_plugin_desc(
                host,
                descriptor.as_raw(),
                Box::new(initializer) as Box<dyn PluginInitializer<'a, P>>,
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
