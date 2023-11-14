use crate::host::Host;
use crate::prelude::*;
use selfie::refs::RefType;
use selfie::Selfie;
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub(crate) struct HostData<'plugin, H>
where
    H: Host,
{
    pub(crate) inner:
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
                let shared = unsafe { &*s.get().cast() };
                ReferentialHostData::new(shared, main_thread(shared))
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
        self.inner.with_referential(|d| d.main_thread().cast())
    }
}

// TODO: move UnsafeCells up
pub(crate) struct ReferentialHostData<'shared, H: Host> {
    shared: &'shared H::Shared<'shared>,
    pub(crate) main_thread: UnsafeCell<H::MainThread<'shared>>,
    audio_processor: Option<UnsafeCell<H::AudioProcessor<'shared>>>,
}

impl<'shared, H: Host> ReferentialHostData<'shared, H> {
    #[inline]
    pub(crate) fn new(
        shared: &'shared H::Shared<'shared>,
        main_thread: H::MainThread<'shared>,
    ) -> Self {
        Self {
            shared,
            main_thread: UnsafeCell::new(main_thread),
            audio_processor: None,
        }
    }

    #[inline]
    pub(crate) fn main_thread(&self) -> NonNull<H::MainThread<'shared>> {
        // SAFETY: the pointer comes from UnsafeCell, it cannot be null
        unsafe { NonNull::new_unchecked(self.main_thread.get().cast()) }
    }

    #[inline]
    pub(crate) fn audio_processor(&self) -> Option<NonNull<H::AudioProcessor<'_>>> {
        self.audio_processor
            .as_ref()
            // SAFETY: pointer cannot be null as it comes from ce cell
            .map(|cell| unsafe { NonNull::new_unchecked(cell.get().cast()) })
    }

    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        self.audio_processor.is_some()
    }

    pub(crate) fn set_new_audio_processor<
        FA: FnOnce(
            &'shared H::Shared<'shared>,
            &mut H::MainThread<'shared>,
        ) -> H::AudioProcessor<'shared>,
    >(
        &mut self,
        audio_processor: FA,
    ) -> Result<(), HostError> {
        match &mut self.audio_processor {
            Some(_) => Err(HostError::AlreadyActivatedPlugin),
            None => {
                self.audio_processor = Some(UnsafeCell::new(audio_processor(
                    self.shared,
                    self.main_thread.get_mut(),
                )));
                Ok(())
            }
        }
    }

    pub(crate) fn remove_audio_processor(
        &mut self,
    ) -> Result<H::AudioProcessor<'shared>, HostError> {
        self.audio_processor
            .take()
            .map(|cell| cell.into_inner())
            .ok_or(HostError::DeactivatedPlugin)
    }
}

pub(crate) struct ReferentialHostDataRef<H>(PhantomData<H>);

impl<'shared, H: Host> RefType<'shared> for ReferentialHostDataRef<H> {
    type Ref = ReferentialHostData<'shared, H>;
}

pub(crate) struct HostDataRef<H>(PhantomData<H>);

impl<'plugin, H: Host> RefType<'plugin> for HostDataRef<H> {
    type Ref = HostData<'plugin, H>;
}
