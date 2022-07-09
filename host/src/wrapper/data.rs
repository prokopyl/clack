use crate::host::Host;
use selfie::refs::RefType;
use selfie::Selfie;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct HostData<'plugin, H>
where
    H: for<'shared> Host<'shared>,
{
    inner:
        Selfie<'plugin, Box<UnsafeCell<<H as Host<'plugin>>::Shared>>, ReferentialHostDataRef<H>>,
}

impl<'a, H: for<'b> Host<'b>> HostData<'a, H> {
    pub fn new<FH>(shared: <H as Host<'a>>::Shared, main_thread: FH) -> Self
    where
        FH: for<'s> FnOnce(&'s <H as Host<'s>>::Shared) -> <H as Host<'s>>::MainThread,
    {
        Self {
            inner: Selfie::new(Box::pin(UnsafeCell::new(shared)), |s| {
                // SAFETY: TODO
                ReferentialHostData::new(main_thread(unsafe { &*s.get().cast() }))
            }),
        }
    }

    #[inline]
    pub fn shared(&self) -> NonNull<<H as Host<'a>>::Shared> {
        // SAFETY: Pointer is from the UnsafeCell, which cannot be null
        unsafe { NonNull::new_unchecked(self.inner.owned().get()) }
    }

    #[inline]
    pub fn main_thread(&self) -> NonNull<<H as Host>::MainThread> {
        self.inner.with_referential(|d| d.main_thread())
    }

    #[inline]
    pub fn audio_processor(&self) -> Option<NonNull<<H as Host>::AudioProcessor>> {
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
            &'s <H as Host<'s>>::Shared,
            &mut <H as Host<'s>>::MainThread,
        ) -> <H as Host<'s>>::AudioProcessor,
    {
        self.inner.with_referential(|d| unsafe {
            // SAFETY: TODO
            let previous = d.replace_audio_processor(Some(audio_processor(
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
            <H as Host<'s>>::AudioProcessor,
            &mut <H as Host<'s>>::MainThread,
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
struct ReferentialHostData<'shared, H: Host<'shared>> {
    main_thread: UnsafeCell<H::MainThread>,
    audio_processor: UnsafeCell<Option<H::AudioProcessor>>,
}

impl<'shared, H: Host<'shared>> ReferentialHostData<'shared, H> {
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

        data.as_ref().map(NonNull::from)
    }

    #[inline]
    unsafe fn replace_audio_processor(
        &self,
        audio_processor: Option<H::AudioProcessor>,
    ) -> Option<H::AudioProcessor> {
        core::mem::replace(&mut *self.audio_processor.get(), audio_processor)
    }
}

struct ReferentialHostDataRef<H>(PhantomData<H>);

impl<'shared, H: Host<'shared>> RefType<'shared> for ReferentialHostDataRef<H> {
    type Ref = ReferentialHostData<'shared, H>;
}

pub struct HostDataRef<H>(PhantomData<H>);

impl<'plugin, H: for<'a> Host<'a>> RefType<'plugin> for HostDataRef<H> {
    type Ref = HostData<'plugin, H>;
}
