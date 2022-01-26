use crate::entry::PluginEntry;
use crate::extensions::HostExtensions;
use crate::host::{HostShared, PluginHoster, SharedHoster};
use crate::instance::PluginAudioConfiguration;
use crate::plugin::{PluginMainThread, PluginShared};
use clap_sys::host::clap_host;
use clap_sys::plugin::{clap_plugin, clap_plugin_entry};
use clap_sys::version::CLAP_VERSION;
use std::cell::UnsafeCell;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
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

pub struct HostWrapper<'a, H: PluginHoster<'a>> {
    shared: H::Shared,
    main_thread: MaybeUninit<UnsafeCell<H>>,
    audio_processor: Option<UnsafeCell<H::AudioProcessor>>,

    _host_info: Arc<HostShared>,
    _host_descriptor: clap_host,

    instance: *mut clap_sys::plugin::clap_plugin,
    _lifetime: PhantomData<&'a clap_plugin_entry>,
}

impl<'a, H: PluginHoster<'a>> HostWrapper<'a, H> {
    pub(crate) fn new<FH, FS>(
        main_thread: FH,
        shared: FS,
        entry: &PluginEntry<'a>,
        plugin_id: &[u8],
        host_info: Arc<HostShared>,
    ) -> Result<Pin<Arc<Self>>, HostError>
    where
        FS: FnOnce() -> H::Shared,
        FH: FnOnce(&'a H::Shared) -> H,
    {
        let mut host_descriptor = clap_host {
            clap_version: CLAP_VERSION,
            host_data: ::core::ptr::null_mut(),
            name: ::core::ptr::null_mut(),
            vendor: ::core::ptr::null_mut(),
            url: ::core::ptr::null_mut(),
            version: ::core::ptr::null_mut(),
            get_extension: get_extension::<H>,
            request_restart: request_restart::<H>,
            request_process: request_process::<H>,
            request_callback: request_callback::<H>,
        };

        host_info.info().write_to_raw(&mut host_descriptor);

        let mut wrapper = Arc::new(Self {
            _host_info: host_info,
            _host_descriptor: host_descriptor,
            shared: shared(),
            main_thread: MaybeUninit::uninit(),
            instance: ::core::ptr::null_mut(),
            audio_processor: None,
            _lifetime: PhantomData,
        });

        let mutable = Arc::get_mut(&mut wrapper).unwrap();
        mutable._host_descriptor.host_data = mutable as *mut _ as *mut _;
        let shared = unsafe { &*(&mutable.shared as *const _) };
        mutable.main_thread = MaybeUninit::new(UnsafeCell::new(main_thread(shared)));
        mutable.instance = unsafe {
            entry
                .instantiate(plugin_id, &mutable._host_descriptor)
                .ok_or(HostError::PluginEntryNotFound)?
                .as_mut()
        };

        mutable
            .shared
            .instantiated(PluginShared::new(mutable.instance));
        unsafe { mutable.main_thread.assume_init_mut() }
            .get_mut()
            .instantiated(PluginMainThread::new(mutable.instance));

        unsafe { Ok(Pin::new_unchecked(wrapper)) }
    }

    pub(crate) fn activate(
        self: Pin<&mut Self>,
        audio_processor: H::AudioProcessor,
        configuration: PluginAudioConfiguration,
    ) -> Result<(), HostError> {
        if self.audio_processor.is_some() {
            return Err(HostError::AlreadyActivatedPlugin);
        }

        // SAFETY: we are never moving out anything but the audio processor
        let mutable = unsafe { Pin::get_unchecked_mut(self) };

        mutable.audio_processor = Some(UnsafeCell::new(audio_processor));

        let success = unsafe {
            println!("{}", mutable._host_descriptor.clap_version.major);
            ((*mutable.instance).activate)(
                mutable.instance,
                configuration.sample_rate,
                *configuration.frames_count_range.start(),
                *configuration.frames_count_range.end(),
            )
        };

        if !success {
            mutable.audio_processor = None;
            return Err(HostError::ActivationFailed);
        }

        Ok(())
    }

    pub(crate) fn deactivate(self: Pin<&mut Self>) -> Result<(), HostError> {
        // SAFETY: we are never moving out anything but the audio processor
        let unpinned = unsafe { Pin::get_unchecked_mut(self) };

        if unpinned.audio_processor.take().is_some() {
            Ok(())
        } else {
            Err(HostError::DeactivatedPlugin)
        }
    }

    pub(crate) unsafe fn start_processing(&self) -> Result<(), HostError> {
        if (self.raw_instance().start_processing)(self.instance) {
            Ok(())
        } else {
            Err(HostError::StartProcessingFailed)
        }
    }

    pub(crate) unsafe fn stop_processing(&self) {
        (self.raw_instance().stop_processing)(self.instance)
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
        NonNull::new_unchecked(self.main_thread.assume_init_ref().get())
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
    pub unsafe fn audio_processor(&self) -> Result<NonNull<H::AudioProcessor>, HostError> {
        self.audio_processor
            .as_ref()
            .map(|p| NonNull::new_unchecked(p.get()))
            .ok_or(HostError::DeactivatedPlugin)
    }

    #[inline]
    pub fn shared(&self) -> &H::Shared {
        &self.shared
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
        F: FnOnce(&HostWrapper<'a, H>) -> Result<T, HostWrapperError>,
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
        F: FnOnce(&HostWrapper<'a, H>) -> Result<T, HostWrapperError>,
    {
        let plugin = Self::from_raw(host)?;

        panic::catch_unwind(AssertUnwindSafe(|| handler(plugin)))
            .map_err(|_| HostWrapperError::Panic)?
    }

    unsafe fn from_raw(raw: *const clap_host) -> Result<&'a Self, HostWrapperError> {
        raw.as_ref()
            .ok_or(HostWrapperError::NullHostInstance)?
            .host_data
            .cast::<HostWrapper<'a, H>>()
            .as_ref()
            .ok_or(HostWrapperError::NullHostData)
    }
}

impl<'a, H: PluginHoster<'a>> Drop for HostWrapper<'a, H> {
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
    InstantiationFailed,
}
// TODO: impl error

unsafe extern "C" fn get_extension<'a, H: PluginHoster<'a>>(
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

unsafe extern "C" fn request_restart<'a, H: PluginHoster<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_restart();
        Ok(())
    });
}

unsafe extern "C" fn request_process<'a, H: PluginHoster<'a>>(host: *const clap_host) {
    HostWrapper::<H>::handle(host, |h| {
        h.shared().request_process();
        Ok(())
    });
}

unsafe extern "C" fn request_callback<'a, H: PluginHoster<'a>>(host: *const clap_host) {
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
