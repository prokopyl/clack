use crate::extensions::wrapper::{PluginWrapper, PluginWrapperError};
use crate::extensions::PluginExtensions;
use crate::host::{HostInfo, HostMainThreadHandle, HostSharedHandle};
use crate::plugin::instance::WrapperData::*;
use crate::plugin::{Plugin, PluginAudioProcessor, PluginError, PluginMainThread};
use crate::prelude::PluginDescriptor;
use crate::process::{Audio, Events, PluginAudioConfiguration, Process};
use clap_sys::plugin::{clap_plugin, clap_plugin_descriptor};
use clap_sys::process::{clap_process, clap_process_status, CLAP_PROCESS_ERROR};
use core::ffi::c_void;
use std::cell::UnsafeCell;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU8, Ordering};

pub(crate) trait PluginInitializer<'a, P: Plugin>: 'a {
    fn init(
        self: Box<Self>,
        host: HostMainThreadHandle<'a>,
    ) -> Result<PluginWrapper<P>, PluginError>;
}

impl<'a, P: Plugin, FS: 'a, FM: 'a> PluginInitializer<'a, P> for (FS, FM)
where
    FS: FnOnce(HostSharedHandle<'a>) -> Result<P::Shared<'a>, PluginError>,
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
        let shared_handle = host.shared();
        let shared = Box::pin(shared_initializer(shared_handle)?);

        // SAFETY: this lives long enough
        let shared_ref = unsafe { &*(shared.as_ref().get_ref() as *const _) };
        let main_thread = main_thread_initializer(host, shared_ref)?;

        // SAFETY: we just created the shared and main_thread together
        Ok(unsafe { PluginWrapper::new(shared_handle, shared, main_thread) })
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
        let shared_handle = host.shared();
        let (shared, main_thread_initializer) = self(host)?;
        let shared = Box::pin(shared);

        // SAFETY: this lives long enough
        let shared_ref = unsafe { &*(shared.as_ref().get_ref() as *const _) };
        let main_thread = main_thread_initializer(shared_ref)?;

        // SAFETY: we just created the shared and main_thread together
        Ok(unsafe { PluginWrapper::new(shared_handle, shared, main_thread) })
    }
}

pub(crate) enum WrapperData<'a, P: Plugin> {
    Initialized(PluginWrapper<'a, P>),
    Initializing,
    Uninitialized(Box<dyn PluginInitializer<'a, P>>),
}

pub(crate) struct PluginBoxInner<'a, P: Plugin> {
    host: HostSharedHandle<'a>,
    state: AtomicU8,
    plugin_data: UnsafeCell<WrapperData<'a, P>>,
}

const UNINITIALIZED: u8 = 0;
const INITIALIZED: u8 = 1;
const INITIALIZING: u8 = 2;
const INITIALIZATION_FAILED: u8 = 3;
const DESTROYING: u8 = 4;

impl<'a, P: Plugin> PluginBoxInner<'a, P> {
    #[inline]
    pub(crate) fn wrapper_uninit(
        &self,
    ) -> Result<Option<&PluginWrapper<'a, P>>, PluginWrapperError> {
        match self.state.load(Ordering::Acquire) {
            INITIALIZING => Ok(None),
            UNINITIALIZED => Err(PluginWrapperError::UninitializedPlugin),
            INITIALIZATION_FAILED => Err(PluginWrapperError::InitializationAlreadyFailed),
            DESTROYING => Err(PluginWrapperError::Destroying),

            // SAFETY: when in the initialized state, it is guarantee that plugin_data is never written to again.
            INITIALIZED => match unsafe { &*self.plugin_data.get() } {
                Initialized(w) => Ok(Some(w)),
                // If the state is Initialized, then the enum should have been set properly
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    #[inline]
    pub(crate) fn wrapper(&self) -> Result<&PluginWrapper<'a, P>, PluginWrapperError> {
        match self.state.load(Ordering::Acquire) {
            INITIALIZING => Err(PluginWrapperError::InitializationAlreadyFailed),
            UNINITIALIZED => Err(PluginWrapperError::UninitializedPlugin),
            INITIALIZATION_FAILED => Err(PluginWrapperError::InitializationAlreadyFailed),
            DESTROYING => Err(PluginWrapperError::Destroying),

            // SAFETY: when in the initialized state, it is guarantee that plugin_data is never written to again.
            INITIALIZED => match unsafe { &*self.plugin_data.get() } {
                Initialized(w) => Ok(w),
                // If the state is Initialized, then the enum should have been set properly
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

impl<'a, P: Plugin> PluginBoxInner<'a, P> {
    fn get_plugin_desc(
        host: HostSharedHandle<'a>,
        desc: &'a clap_plugin_descriptor,
        initializer: Box<dyn PluginInitializer<'a, P>>,
    ) -> clap_plugin {
        clap_plugin {
            desc,
            plugin_data: Box::into_raw(Box::new(Self {
                host,
                plugin_data: UnsafeCell::new(Uninitialized(initializer)),
                state: AtomicU8::new(UNINITIALIZED),
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
    pub fn host(&self) -> &HostSharedHandle<'a> {
        &self.host
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn init(plugin: *const clap_plugin) -> bool {
        PluginWrapper::<P>::handle_plugin_data(plugin, |data| {
            // We can only keep a &mut reference in here, until init() is called.
            let data = data.as_ref();

            let current_state = data.state.compare_exchange(
                UNINITIALIZED,
                INITIALIZING,
                Ordering::SeqCst,
                Ordering::Acquire,
            );

            let uninit_data = match current_state {
                Ok(UNINITIALIZED) => data.plugin_data.get().replace(Initializing),
                Ok(s) | Err(s) => match s {
                    INITIALIZED => return Err(PluginWrapperError::AlreadyInitialized),
                    DESTROYING => return Err(PluginWrapperError::Destroying),
                    INITIALIZATION_FAILED | INITIALIZING => {
                        return Err(PluginWrapperError::InitializationAlreadyFailed)
                    }
                    _ => unreachable!(),
                },
            };

            let Uninitialized(initializer) = uninit_data else {
                unreachable!()
            };

            let init_result = initializer.init(data.host.as_main_thread_unchecked());

            match init_result {
                Ok(wrapper) => {
                    // We now guaranteed that the current state is INITIALIZING, so there is nothing to drop.
                    data.plugin_data.get().write(Initialized(wrapper));
                    // The write operation completed, we can now inform other threads that initialization is complete.
                    data.state.store(INITIALIZED, Ordering::Release);
                    Ok(())
                }
                Err(e) => {
                    data.state.store(INITIALIZATION_FAILED, Ordering::Release);
                    Err(e.into())
                }
            }
        })
        .is_some()
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        // Deactivate the plugin, in case the host didn't call deactivate() first.
        PluginWrapper::<P>::handle(plugin, |p| {
            if p.is_active() {
                p.deactivate()
            } else {
                Ok(())
            }
        });

        PluginWrapper::<P>::handle_plugin_data(plugin, |data| {
            data.as_ref().state.store(DESTROYING, Ordering::SeqCst);

            let _ = Box::from_raw(data.as_ptr());
            Ok(())
        });

        if let Some(plugin) = plugin.cast_mut().as_mut() {
            // Try our best to guard against double-free
            if plugin.plugin_data.is_null() {
                return;
            }

            // This allows destroy() to be called twice safely: all PluginWrapper methods immediately
            // fail if plugin_data is null, and we check it against above before deallocating.
            plugin.plugin_data = core::ptr::null_mut();

            let _ = Box::from_raw(plugin as *mut clap_plugin);
        }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn activate(
        plugin: *const clap_plugin,
        sample_rate: f64,
        min_sample_count: u32,
        max_sample_count: u32,
    ) -> bool {
        PluginWrapper::<P>::handle(plugin, |p| {
            let config = PluginAudioConfiguration {
                sample_rate,
                min_frames_count: min_sample_count,
                max_frames_count: max_sample_count,
            };

            p.activate(config)
        })
        .is_some()
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn deactivate(plugin: *const clap_plugin) {
        PluginWrapper::<P>::handle(plugin, |p| p.deactivate());
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn reset(plugin: *const clap_plugin) {
        PluginWrapper::<P>::handle(plugin, |p| {
            p.audio_processor()?.as_mut().reset();
            Ok(())
        });
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn start_processing(plugin: *const clap_plugin) -> bool {
        PluginWrapper::<P>::handle(plugin, |p| {
            Ok(p.audio_processor()?.as_mut().start_processing()?)
        })
        .is_some()
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn stop_processing(plugin: *const clap_plugin) {
        PluginWrapper::<P>::handle(plugin, |p| {
            p.audio_processor()?.as_mut().stop_processing();
            Ok(())
        });
    }

    #[allow(clippy::missing_safety_doc)]
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

    #[allow(clippy::missing_safety_doc)]
    unsafe extern "C" fn get_extension(
        plugin: *const clap_plugin,
        identifier: *const std::os::raw::c_char,
    ) -> *const c_void {
        let identifier = CStr::from_ptr(identifier);
        let mut builder = PluginExtensions::new(identifier);

        PluginWrapper::<P>::handle_plugin_data(plugin, |data| {
            let p = data.as_ref().wrapper_uninit()?;
            P::declare_extensions(&mut builder, p.map(|p| p.shared()));
            Ok(())
        });
        builder.found()
    }

    #[allow(clippy::missing_safety_doc)]
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
        shared_initializer: impl FnOnce(HostSharedHandle<'a>) -> Result<P::Shared<'a>, PluginError> + 'a,
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
    /// struct MyPluginShared<'a>(HostSharedHandle<'a>, Sender<u32>);
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
            // SAFETY: the destroy fn is valid as it's provided by us directly.
            unsafe { destroy(self.inner.as_ref()) }
        }
    }
}
