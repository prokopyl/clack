use crate::host::Host;
use crate::instance::handle::PluginAudioProcessorHandle;
use clap_sys::plugin::clap_plugin;
use selfie::refs::RefType;
use selfie::Selfie;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct HostData<'plugin, H>
where
    H: Host,
{
    inner:
        Selfie<'plugin, Box<UnsafeCell<<H as Host>::Shared<'plugin>>>, ReferentialHostDataRef<H>>,
}

impl<'a, H: Host> HostData<'a, H> {
    pub fn new<FH>(shared: <H as Host>::Shared<'a>, main_thread: FH) -> Self
    where
        FH: for<'s> FnOnce(&'s <H as Host>::Shared<'s>) -> <H as Host>::MainThread<'s>,
    {
        Self {
            inner: Selfie::new(Box::pin(UnsafeCell::new(shared)), |s| {
                // SAFETY: TODO
                ReferentialHostData::new(main_thread(unsafe { &*s.get().cast() }))
            }),
        }
    }

    #[inline]
    pub fn shared(&self) -> NonNull<<H as Host>::Shared<'a>> {
        // SAFETY: Pointer is from the UnsafeCell, which cannot be null
        unsafe { NonNull::new_unchecked(self.inner.owned().get()) }
    }

    #[inline]
    pub fn main_thread(&self) -> NonNull<<H as Host>::MainThread<'_>> {
        self.inner.with_referential(|d| d.main_thread())
    }

    #[inline]
    pub fn audio_processor(&self) -> Option<NonNull<<H as Host>::AudioProcessor<'_>>> {
        self.inner.with_referential(|d| d.audio_processor())
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.audio_processor().is_some()
    }

    #[inline]
    pub fn activate<FA>(&self, audio_processor: FA, instance: &clap_plugin)
    where
        FA: for<'s> FnOnce(
            PluginAudioProcessorHandle<'s>,
            &'s <H as Host>::Shared<'s>,
            &mut <H as Host>::MainThread<'s>,
        ) -> <H as Host>::AudioProcessor<'s>,
    {
        self.inner.with_referential(|d| unsafe {
            // SAFETY: TODO
            let previous = d.replace_audio_processor(Some(audio_processor(
                PluginAudioProcessorHandle::new(instance as *const _ as *mut _),
                self.shared().cast().as_ref(),
                self.main_thread().as_mut(),
            )));

            if previous.is_some() {
                panic!("Tried to enable an already enabled audio processor")
            }
        })
    }

    #[inline]
    pub fn deactivate<T>(
        &self,
        drop: impl for<'s> FnOnce(
            <H as Host>::AudioProcessor<'s>,
            &mut <H as Host>::MainThread<'s>,
        ) -> T,
    ) -> T {
        self.inner.with_referential(|d| unsafe {
            let main_thread = &mut *d.main_thread.get();
            // PANIC: should be checked by caller
            drop(d.replace_audio_processor(None).unwrap(), main_thread)
        })
    }
}

// TODO: move UnsafeCells up
struct ReferentialHostData<'shared, H: Host> {
    main_thread: UnsafeCell<H::MainThread<'shared>>,
    audio_processor: UnsafeCell<Option<H::AudioProcessor<'shared>>>,
}

impl<'shared, H: Host> ReferentialHostData<'shared, H> {
    #[inline]
    fn new(main_thread: H::MainThread<'shared>) -> Self {
        Self {
            main_thread: UnsafeCell::new(main_thread),
            audio_processor: UnsafeCell::new(None),
        }
    }

    #[inline]
    fn main_thread(&self) -> NonNull<H::MainThread<'shared>> {
        // SAFETY: the pointer comes from UnsafeCell, it cannot be null
        unsafe { NonNull::new_unchecked(self.main_thread.get()) }
    }

    #[inline]
    fn audio_processor(&self) -> Option<NonNull<H::AudioProcessor<'shared>>> {
        // SAFETY: &self guarantees at least shared access to the outer Option
        let data = unsafe { &*self.audio_processor.get() };

        data.as_ref().map(NonNull::from)
    }

    #[inline]
    unsafe fn replace_audio_processor(
        &self,
        audio_processor: Option<H::AudioProcessor<'shared>>,
    ) -> Option<H::AudioProcessor<'shared>> {
        core::mem::replace(&mut *self.audio_processor.get(), audio_processor)
    }
}

struct ReferentialHostDataRef<H>(PhantomData<H>);

impl<'shared, H: Host> RefType<'shared> for ReferentialHostDataRef<H> {
    type Ref = ReferentialHostData<'shared, H>;
}

pub struct HostDataRef<H>(PhantomData<H>);

impl<'plugin, H: Host> RefType<'plugin> for HostDataRef<H> {
    type Ref = HostData<'plugin, H>;
}
