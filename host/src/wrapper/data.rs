use crate::host::PluginHoster;
use selfie::refs::RefType;
use selfie::Selfie;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct HostData<'plugin, H>
where
    H: for<'shared> PluginHoster<'shared>,
{
    inner: Selfie<
        'plugin,
        Box<UnsafeCell<<H as PluginHoster<'plugin>>::Shared>>,
        ReferentialHostDataRef<H>,
    >,
}

impl<'a, H: for<'b> PluginHoster<'b>> HostData<'a, H> {
    pub fn new<FH>(shared: <H as PluginHoster<'a>>::Shared, main_thread: FH) -> Self
    where
        FH: for<'s> FnOnce(
            &'s <H as PluginHoster<'s>>::Shared,
        ) -> <H as PluginHoster<'s>>::MainThread,
    {
        Self {
            inner: Selfie::new(Box::pin(UnsafeCell::new(shared)), |s| {
                // SAFETY: TODO
                ReferentialHostData::new(main_thread(unsafe { &*s.get().cast() }))
            }),
        }
    }

    #[inline]
    pub fn shared(&self) -> &<H as PluginHoster<'a>>::Shared {
        unsafe { &*self.inner.owned().get() }
    }

    #[inline]
    pub fn shared_raw(&self) -> NonNull<<H as PluginHoster<'a>>::Shared> {
        // SAFETY: Pointer is from the UnsafeCell, which cannot be null
        unsafe { NonNull::new_unchecked(self.inner.owned().get()) }
    }

    #[inline]
    pub fn main_thread(&self) -> NonNull<<H as PluginHoster>::MainThread> {
        self.inner.with_referential(|d| d.main_thread())
    }

    #[inline]
    pub fn audio_processor(&self) -> Option<NonNull<<H as PluginHoster>::AudioProcessor>> {
        self.inner.with_referential(|d| d.audio_processor())
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.audio_processor().is_some()
    }

    #[inline]
    pub fn activate<FA>(&self, audio_processor: FA)
    where
        FA: for<'s> FnOnce(
            &'s <H as PluginHoster<'s>>::Shared,
            &mut <H as PluginHoster<'s>>::MainThread,
        ) -> <H as PluginHoster<'s>>::AudioProcessor,
    {
        self.inner.with_referential(|d| unsafe {
            // SAFETY: TODO
            d.set_audio_processor(Some(audio_processor(
                self.shared_raw().cast().as_ref(),
                self.main_thread().as_mut(),
            )))
        })
    }

    #[inline]
    pub fn deactivate(&self) {
        self.inner
            .with_referential(|d| unsafe { d.set_audio_processor(None) })
    }
}

struct ReferentialHostData<'shared, H: PluginHoster<'shared>> {
    main_thread: UnsafeCell<H::MainThread>,
    audio_processor: UnsafeCell<Option<H::AudioProcessor>>,
}

impl<'shared, H: PluginHoster<'shared>> ReferentialHostData<'shared, H> {
    #[inline]
    fn new(main_thread: H::MainThread) -> Self {
        Self {
            main_thread: UnsafeCell::new(main_thread),
            audio_processor: UnsafeCell::new(None),
        }
    }

    #[inline]
    fn main_thread(&self) -> NonNull<H::MainThread> {
        // SAFETY: the pointer comes from UnsafeCell, it cannot be null
        unsafe { NonNull::new_unchecked(self.main_thread.get()) }
    }

    #[inline]
    fn audio_processor(&self) -> Option<NonNull<H::AudioProcessor>> {
        // SAFETY: &self guarantees at least shared access to the outer Option
        let data = unsafe { &*self.audio_processor.get() };

        data.as_ref().map(|a| NonNull::from(a))
    }

    #[inline]
    unsafe fn set_audio_processor(&self, audio_processor: Option<H::AudioProcessor>) {
        *&mut *self.audio_processor.get() = audio_processor
    }
}

struct ReferentialHostDataRef<H>(PhantomData<H>);

impl<'shared, H: PluginHoster<'shared>> RefType<'shared> for ReferentialHostDataRef<H> {
    type Ref = ReferentialHostData<'shared, H>;
}

pub struct HostDataRef<H>(PhantomData<H>);

impl<'plugin, H: for<'a> PluginHoster<'a>> RefType<'plugin> for HostDataRef<H> {
    type Ref = HostData<'plugin, H>;
}
