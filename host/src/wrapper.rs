use crate::host::{MainThreadHoster, PluginHoster, SharedHoster};
use crate::plugin::{PluginMainThreadHandle, PluginSharedHandle};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use selfie::Selfie;
use std::cell::{Cell, UnsafeCell};
use std::fmt;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::ptr::NonNull;

mod panic {
    #[cfg(not(test))]
    #[allow(unused)]
    pub use std::panic::catch_unwind;

    #[cfg(test)]
    #[inline]
    #[allow(unused)]
    pub fn catch_unwind<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
        f: F,
    ) -> std::thread::Result<R> {
        Ok(f())
    }
}

pub(crate) mod instance;
use instance::*;

pub(crate) mod descriptor;

pub(crate) mod data;
use data::*;

// Self-referential lifetimes on the cheap
struct HosterWrapper<'a, H: PluginHoster<'a>> {
    shared: <H as PluginHoster<'a>>::Shared,
    main_thread: MaybeUninit<UnsafeCell<<H as PluginHoster<'a>>::MainThread>>,
    audio_processor: Option<UnsafeCell<<H as PluginHoster<'a>>::AudioProcessor>>,
}

struct HosterWrapperToken<H>(PhantomData<H>);

impl<'a, H: PluginHoster<'a>> selfie::refs::RefType<'a> for HosterWrapperToken<H> {
    type Ref = HosterWrapper<'a, H>;
}

struct PluginInstance(Cell<*mut clap_plugin>);
// impl Unpin for PluginInstance {}

impl Deref for PluginInstance {
    type Target = clap_plugin;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

// Referential structure:
// instance -> clap_host -> host_shared & main_thread & audio_processor -> shared -> Option<instance>

// shared optionally references the plugin instance
// main_thread and audio_processor reference both shared and optionally the plugin instance
// clap_host references global host data (name etc.), as well as main_thread and audio_processor
// the plugin instance references clap_host
// all other fields just need to be kept around at a stable location

pub struct HostWrapper<H: for<'a> PluginHoster<'a>> {
    data: Selfie<'static, RawPluginInstanceRef, HostDataRef<H>>,
    // FIXME: this is awful
    //hoster: Selfie<'static, &'static (), HosterWrapperToken<H>>,
    /*shared: <H as PluginHoster<'static>>::Shared,
    main_thread: MaybeUninit<UnsafeCell<H>>,
    audio_processor: Option<UnsafeCell<<H as PluginHoster<'static>>::AudioProcessor>>,*/
    /*_host_info: Arc<HostShared>,
    _host_descriptor: clap_host,

    instance: *mut clap_plugin,
    _bundle: PluginBundle,*/
}

// SAFETY: The only non-thread-safe methods on this type are unsafe
unsafe impl<H: for<'h> PluginHoster<'h>> Send for HostWrapper<H> {}
unsafe impl<H: for<'h> PluginHoster<'h>> Sync for HostWrapper<H> {}

impl<H: for<'h> PluginHoster<'h>> HostWrapper<H> {
    pub(crate) fn new<FS, FH>(shared: FS, main_thread: FH) -> Self
    where
        FS: for<'s> FnOnce(&'s ()) -> <H as PluginHoster<'s>>::Shared,
        FH: for<'s> FnOnce(
            &'s <H as PluginHoster<'s>>::Shared,
        ) -> <H as PluginHoster<'s>>::MainThread,
    {
        let instance_ptr = Pin::new(RawPluginInstanceRef::default());

        Self {
            data: Selfie::new(instance_ptr, |_| {
                HostData::new(shared(&()), |s| main_thread(s))
            }),
        }
    }

    pub(crate) fn instantiated(&self, instance: *mut clap_plugin) {
        self.data.with_referential(|d| {
            // SAFETY: TODO?
            unsafe { d.shared_raw().as_mut() }.instantiated(PluginSharedHandle::new(instance));
            unsafe { d.main_thread().as_mut() }.instantiated(PluginMainThreadHandle::new(instance));
        });
    }

    #[inline]
    pub(crate) unsafe fn activate<FA>(&self, audio_processor: FA)
    where
        FA: for<'a> FnOnce(
            &'a <H as PluginHoster<'a>>::Shared,
            &mut <H as PluginHoster<'a>>::MainThread,
        ) -> <H as PluginHoster<'a>>::AudioProcessor,
    {
        self.data.with_referential(|d| d.activate(audio_processor))
    }

    #[inline]
    pub(crate) unsafe fn deactivate(&self) {
        self.data.with_referential(|d| d.deactivate())
    }

    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        self.data.with_referential(|d| d.is_active())
    }

    /// Returns a raw, non-null pointer to the host's main thread ([`PluginHoster`](crate::host::PluginHoster))
    /// struct.
    ///
    /// # Safety
    /// The caller must ensure this method is only called on the main thread.
    ///
    /// The pointer is safe to mutably dereference, as long as the caller ensures it is not being
    /// aliased, as per usual safety rules.
    #[inline]
    pub unsafe fn main_thread(&self) -> NonNull<<H as PluginHoster>::MainThread> {
        self.data.with_referential(|d| d.main_thread().cast())
    }

    /// Returns a raw, non-null pointer to the host's ([`AudioProcessor`](crate::host::PluginHoster::AudioProcessor))
    /// struct.
    ///
    /// # Safety
    /// The caller must ensure this method is only called on the audio thread.
    ///
    /// The pointer is safe to mutably dereference, as long as the caller ensures it is not being
    /// aliased, as per usual safety rules.
    #[inline]
    pub unsafe fn audio_processor(
        &self,
    ) -> Result<NonNull<<H as PluginHoster>::AudioProcessor>, HostError> {
        self.data.with_referential(|d| {
            d.audio_processor()
                .map(|a| a.cast())
                .ok_or(HostError::DeactivatedPlugin)
        })
    }

    #[inline]
    pub fn shared(&self) -> &<H as PluginHoster>::Shared {
        // SAFETY: TODO
        self.data
            .with_referential(|d| unsafe { d.shared_raw().cast().as_ref() })
    }

    /// TODO: docs
    ///
    /// # Safety
    ///
    /// The given host wrapper type `H` **must** be the correct type for the received pointer. Otherwise,
    /// incorrect casts will occur, which will lead to Undefined Behavior.
    ///
    /// The `host` pointer must also point to a valid instance of `clap_host`, as created by
    /// the CLAP Host. While this function does a couple of simple safety checks, only a few common
    /// cases are actually covered (i.e. null checks), and those **must not** be relied upon: those
    /// checks only exist to help debugging.
    pub unsafe fn handle<T, F>(host: *const clap_host, handler: F) -> Option<T>
    where
        F: FnOnce(&HostWrapper<H>) -> Result<T, HostWrapperError>,
    {
        match Self::handle_panic(host, handler) {
            Ok(value) => Some(value),
            Err(_e) => {
                // logging::plugin_log::<P>(host, &e); TODO

                None
            }
        }
    }

    unsafe fn handle_panic<T, F>(host: *const clap_host, handler: F) -> Result<T, HostWrapperError>
    where
        F: FnOnce(&HostWrapper<H>) -> Result<T, HostWrapperError>,
    {
        let plugin = Self::from_raw(host)?;

        panic::catch_unwind(AssertUnwindSafe(|| handler(plugin)))
            .map_err(|_| HostWrapperError::Panic)?
    }

    unsafe fn from_raw<'a>(raw: *const clap_host) -> Result<&'a Self, HostWrapperError> {
        raw.as_ref()
            .ok_or(HostWrapperError::NullHostInstance)?
            .host_data
            .cast::<HostWrapper<H>>()
            .as_ref()
            .ok_or(HostWrapperError::NullHostData)
    }
}
/*
impl<'a, H: for<'h> PluginHoster<'h>> Drop for HostWrapper<H> {
    #[inline]
    fn drop(&mut self) {
        // ((*self.instance).destroy) == core::ptr::null();
        unsafe { ((*self.instance).destroy)(self.instance) }
    }
}*/

#[derive(Debug, Eq, PartialEq)]
pub enum HostError {
    StartProcessingFailed,
    AlreadyActivatedPlugin,
    DeactivatedPlugin,
    ActivationFailed,
    PluginEntryNotFound,
    PluginNotFound,
    MissingPluginFactory,
    InstantiationFailed,
    PluginIdNulError,
    ProcessingFailed,
    ProcessorHandlePoisoned,
    ProcessingStopped,
    ProcessingStarted,
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::StartProcessingFailed => write!(f, "Could not start processing"),
            Self::AlreadyActivatedPlugin => write!(f, "Plugin was already activated"),
            Self::DeactivatedPlugin => write!(f, "Plugin is currently deactivated"),
            Self::ActivationFailed => write!(f, "Unable to activate"),
            Self::PluginEntryNotFound => write!(f, "No entry found for the specified plugin"),
            Self::PluginNotFound => write!(f, "Specified plugin was not found"),
            Self::MissingPluginFactory => write!(f, "No plugin factory was provided"),
            Self::InstantiationFailed => write!(f, "Could not instantiate"),
            Self::PluginIdNulError => write!(f, "Plugin ID was null"),
            Self::ProcessingFailed => write!(f, "Could not process"),
            Self::ProcessorHandlePoisoned => write!(f, "Audio Processor handle was poisoned"),
            Self::ProcessingStopped => write!(f, "Audio Processor is currently stopped"),
            Self::ProcessingStarted => write!(f, "Audio Processor is currently started"),
        }
    }
}

impl std::error::Error for HostError {}

pub enum HostWrapperError {
    NullHostInstance,
    NullHostData,
    Panic,
    HostError(HostError),
}
