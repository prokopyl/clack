use crate::bundle::PluginBundleHandle;
use crate::entry::PluginEntry;
use crate::extensions::HostExtensions;
use crate::factory::PluginFactory;
use crate::host::{HostShared, PluginHoster, SharedHoster};
use crate::instance::PluginAudioConfiguration;
use crate::plugin::{PluginMainThreadHandle, PluginSharedHandle};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::version::CLAP_VERSION;
use selfie::Selfie;
use std::cell::{Cell, UnsafeCell};
use std::ffi::{c_void, CStr};
use std::fmt;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::Arc;

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

// Self-referential lifetimes on the cheap
struct HosterWrapper<'a, H: PluginHoster<'a>> {
    shared: <H as PluginHoster<'a>>::Shared,
    main_thread: MaybeUninit<UnsafeCell<H>>,
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

pub struct HostWrapper<H: for<'a> PluginHoster<'a>> {
    hoster: Selfie<'static, &'static (), HosterWrapperToken<H>>,
    /*shared: <H as PluginHoster<'static>>::Shared,
    main_thread: MaybeUninit<UnsafeCell<H>>,
    audio_processor: Option<UnsafeCell<<H as PluginHoster<'static>>::AudioProcessor>>,*/
    _host_info: Arc<HostShared>,
    _host_descriptor: clap_host,

    instance: *mut clap_plugin,
    _bundle: PluginBundleHandle,
}

// SAFETY: The only non-thread-safe method on this type are unsafe
unsafe impl<'a, H: 'a + for<'h> PluginHoster<'h>> Send for HostWrapper<H> {}
unsafe impl<'a, H: 'a + for<'h> PluginHoster<'h>> Sync for HostWrapper<H> {}

impl<H: for<'h> PluginHoster<'h>> HostWrapper<H> {
    pub(crate) fn new<FH, FS>(
        main_thread: FH,
        shared: FS,
        entry: &PluginEntry,
        plugin_id: &[u8],
        host_info: Arc<HostShared>,
    ) -> Result<Pin<Arc<Self>>, HostError>
    where
        FS: for<'s> FnOnce(&'s ()) -> <H as PluginHoster<'s>>::Shared,
        FH: for<'s> FnOnce(&'s <H as PluginHoster<'s>>::Shared) -> H,
    {
        let mut host_descriptor = clap_host {
            clap_version: CLAP_VERSION,
            host_data: core::ptr::null_mut(),
            name: core::ptr::null_mut(),
            vendor: core::ptr::null_mut(),
            url: core::ptr::null_mut(),
            version: core::ptr::null_mut(),
            get_extension: get_extension::<H>,
            request_restart: request_restart::<H>,
            request_process: request_process::<H>,
            request_callback: request_callback::<H>,
        };

        host_info.info().write_to_raw(&mut host_descriptor);

        let hoster = Selfie::new(Pin::new(&()), |_| HosterWrapper {
            shared: shared(&()),
            main_thread: MaybeUninit::uninit(),
            audio_processor: None,
        });

        let mut wrapper = Arc::new(Self {
            _host_info: host_info,
            _host_descriptor: host_descriptor,
            hoster,
            instance: core::ptr::null_mut(),
            _bundle: entry.bundle.clone(),
        });

        let mutable = Arc::get_mut(&mut wrapper).unwrap();
        mutable._host_descriptor.host_data = mutable as *mut _ as *mut _;
        let shared = unsafe { &*(&mutable.hoster().shared as *const _) };
        mutable.hoster_mut().main_thread = MaybeUninit::new(UnsafeCell::new(main_thread(shared)));
        mutable.instance = unsafe {
            entry
                .get_factory::<PluginFactory>()
                .ok_or(HostError::MissingPluginFactory)?
                .instantiate(plugin_id, &mutable._host_descriptor)?
                .as_mut()
        };

        let instance = mutable.instance;

        mutable
            .hoster_mut()
            .shared
            .instantiated(PluginSharedHandle::new(instance));

        unsafe { mutable.hoster_mut().main_thread.assume_init_mut() }
            .get_mut()
            .instantiated(PluginMainThreadHandle::new(instance));

        unsafe { Ok(Pin::new_unchecked(wrapper)) }
    }

    #[inline]
    fn hoster(&self) -> &HosterWrapper<H> {
        self.hoster
            .with_referential(|h| unsafe { &*(h as *const _) })
    }

    #[inline]
    fn hoster_mut(&mut self) -> &mut HosterWrapper<H> {
        self.hoster
            .with_referential_mut(|h| unsafe { &mut *(h as *mut _) })
    }

    pub(crate) fn activate<FA>(
        self: Pin<&mut Self>,
        audio_processor: FA,
        configuration: PluginAudioConfiguration,
    ) -> Result<(), HostError>
    where
        FA: for<'a> FnOnce(
            &'a <H as PluginHoster<'a>>::Shared,
            &mut H,
        ) -> <H as PluginHoster<'a>>::AudioProcessor,
    {
        if self.hoster().audio_processor.is_some() {
            return Err(HostError::AlreadyActivatedPlugin);
        }

        // SAFETY: we are never moving out anything but the audio processor
        let mutable = unsafe { Pin::get_unchecked_mut(self) };
        let hoster_mut = mutable.hoster_mut();
        let shared = unsafe { &*(&hoster_mut.shared as *const _) };
        let main_thread = unsafe { hoster_mut.main_thread.assume_init_mut().get_mut() };

        hoster_mut.audio_processor = Some(UnsafeCell::new(audio_processor(shared, main_thread)));

        let success = unsafe {
            ((*mutable.instance).activate)(
                mutable.instance,
                configuration.sample_rate,
                *configuration.frames_count_range.start(),
                *configuration.frames_count_range.end(),
            )
        };

        if !success {
            mutable.hoster_mut().audio_processor = None;
            return Err(HostError::ActivationFailed);
        }

        Ok(())
    }

    pub(crate) fn deactivate(self: Pin<&mut Self>) -> Result<(), HostError> {
        // SAFETY: we are never moving out anything but the audio processor
        let unpinned = unsafe { Pin::get_unchecked_mut(self) };

        if unpinned.hoster_mut().audio_processor.take().is_some() {
            Ok(())
        } else {
            Err(HostError::DeactivatedPlugin)
        }
    }

    pub(crate) unsafe fn start_processing(&self) -> Result<(), HostError> {
        if (self.raw_instance().start_processing)(self.instance) {
            return Ok(());
        }

        Err(HostError::StartProcessingFailed)
    }

    pub(crate) unsafe fn stop_processing(&self) {
        (self.raw_instance().stop_processing)(self.instance)
    }

    pub(crate) unsafe fn on_main_thread(&self) {
        (self.raw_instance().on_main_thread)(self.instance)
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
    pub unsafe fn main_thread(&self) -> NonNull<H> {
        NonNull::new_unchecked(self.hoster().main_thread.assume_init_ref().get())
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
        self.hoster()
            .audio_processor
            .as_ref()
            .map(|p| NonNull::new_unchecked(p.get()))
            .ok_or(HostError::DeactivatedPlugin)
    }

    #[inline]
    pub fn shared(&self) -> &<H as PluginHoster>::Shared {
        &self.hoster().shared
    }

    #[inline]
    pub fn raw_instance(&self) -> &clap_plugin {
        // SAFETY: this pointer is always valid once the instance is fully constructed
        unsafe { &*self.instance }
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

impl<'a, H: for<'h> PluginHoster<'h>> Drop for HostWrapper<H> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ((*self.instance).destroy)(self.instance) }
    }
}

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

unsafe extern "C" fn get_extension<H: for<'a> PluginHoster<'a>>(
    host: *const clap_host,
    identifier: *const std::os::raw::c_char,
) -> *const c_void {
    let identifier = CStr::from_ptr(identifier);
    let mut builder = HostExtensions::new(identifier);

    HostWrapper::<H>::handle(host, |h| {
        H::declare_extensions(&mut builder, h.shared());
        Ok(())
    });
    builder.found()
}

unsafe extern "C" fn request_restart<H: for<'a> PluginHoster<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_restart();
        Ok(())
    });
}

unsafe extern "C" fn request_process<H: for<'a> PluginHoster<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_process();
        Ok(())
    });
}

unsafe extern "C" fn request_callback<H: for<'a> PluginHoster<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_callback();
        Ok(())
    });
}

pub enum HostWrapperError {
    NullHostInstance,
    NullHostData,
    Panic,
    HostError(HostError),
}
