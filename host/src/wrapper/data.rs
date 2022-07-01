use crate::host::PluginHoster;
use clap_sys::plugin::clap_plugin;
use selfie::refs::RefType;
use selfie::Selfie;
use stable_deref_trait::StableDeref;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

pub struct ClapInstance(Option<NonNull<clap_plugin>>);

impl Deref for ClapInstance {
    type Target = ();

    #[inline]
    fn deref(&self) -> &Self::Target {
        &()
    }
}

// SAFETY: ClapInstance really doesn't deref to anything
unsafe impl StableDeref for ClapInstance {}

pub struct HostData<'plugin, H: for<'shared> PluginHoster<'shared>> {
    inner: Selfie<'plugin, Box<<H as PluginHoster<'plugin>>::Shared>, ReferentialHostDataRef<H>>,
}

impl<'a, H: for<'b> PluginHoster<'b>> HostData<'a, H> {
    pub fn new<FH>(shared: <H as PluginHoster<'a>>::Shared, main_thread: FH) -> Self
    where
        FH: for<'s> FnOnce(&'s <H as PluginHoster<'a>>::Shared) -> H,
    {
        Self {
            inner: Selfie::new(Box::pin(shared), |s| ReferentialHostData {
                main_thread: main_thread(s),
                audio_processor: None,
            }),
        }
    }

    #[inline]
    pub fn shared(&self) -> &<H as PluginHoster<'a>>::Shared {
        self.inner.owned()
    }

    #[inline]
    pub fn main_thread(&self) -> NonNull<H> {
        self.inner
            .with_referential(|d| NonNull::from(&d.main_thread))
    }

    #[inline]
    pub fn audio_processor(&self) -> Option<NonNull<<H as PluginHoster<'a>>::AudioProcessor>> {
        self.inner
            .with_referential(|d| d.audio_processor.as_ref().map(|a| NonNull::from(a).cast()))
    }
}

struct ReferentialHostData<'shared, H: PluginHoster<'shared>> {
    main_thread: H,
    audio_processor: Option<H::AudioProcessor>,
}

struct ReferentialHostDataRef<H>(PhantomData<H>);

impl<'shared, H: PluginHoster<'shared>> RefType<'shared> for ReferentialHostDataRef<H> {
    type Ref = ReferentialHostData<'shared, H>;
}

struct HostDataRef<H>(PhantomData<H>);

impl<'plugin, H: 'plugin + for<'a> PluginHoster<'a>> RefType<'plugin> for HostDataRef<H> {
    type Ref = HostData<'plugin, H>;
}

pub struct InstanceWithHostData<H: for<'a> PluginHoster<'a>> {
    inner: Selfie<'static, ClapInstance, HostDataRef<H>>,
}
