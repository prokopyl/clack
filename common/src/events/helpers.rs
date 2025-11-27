macro_rules! impl_event_helpers {
    ($raw_type:ty) => {
        #[inline]
        pub const fn as_raw(&self) -> &$raw_type {
            &self.inner
        }

        #[inline]
        pub const fn as_raw_mut(&mut self) -> &mut $raw_type {
            &mut self.inner
        }

        #[inline]
        pub const fn from_raw(raw: &$raw_type) -> Self {
            crate::events::ensure_event_matches::<Self>(&raw.header);

            Self { inner: *raw }
        }

        #[inline]
        pub const fn from_raw_ref(raw: &$raw_type) -> &Self {
            crate::events::ensure_event_matches::<Self>(&raw.header);

            // SAFETY: This type is #[repr(C)]-compatible with $raw_type
            unsafe { &*(raw as *const $raw_type as *const Self) }
        }

        #[inline]
        pub const fn from_raw_mut(raw: &mut $raw_type) -> &mut Self {
            crate::events::ensure_event_matches::<Self>(&raw.header);

            // SAFETY: This type is #[repr(C)]-compatible with $raw_type
            unsafe { &mut *(raw as *mut $raw_type as *mut Self) }
        }
    };
}

pub(crate) use impl_event_helpers;
