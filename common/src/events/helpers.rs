macro_rules! impl_event_helpers {
    ($raw_type:ty) => {
        /// Returns a shared reference to the underlying raw, C-FFI compatible event struct.
        #[inline]
        pub const fn as_raw(&self) -> &$raw_type {
            &self.inner
        }

        /// Returns a mutable reference to the underlying raw, C-FFI compatible event struct.
        #[inline]
        pub const fn as_raw_mut(&mut self) -> &mut $raw_type {
            &mut self.inner
        }

        /// Creates a new event of this type from a reference to a raw, C-FFI compatible event
        /// struct.
        ///
        /// # Panics
        ///
        /// This method will panic if the given event struct's header doesn't actually match
        /// the expected event type.
        #[inline]
        pub const fn from_raw(raw: &$raw_type) -> Self {
            crate::events::ensure_event_matches::<Self>(&raw.header);

            Self { inner: *raw }
        }

        /// Creates a reference to an event of this type from a reference to a raw,
        /// C-FFI compatible event struct.
        ///
        /// # Panics
        ///
        /// This method will panic if the given event struct's header doesn't actually match
        /// the expected event type.
        #[inline]
        pub const fn from_raw_ref(raw: &$raw_type) -> &Self {
            crate::events::ensure_event_matches::<Self>(&raw.header);

            // SAFETY: This type is #[repr(C)]-compatible with $raw_type
            unsafe { &*(raw as *const $raw_type as *const Self) }
        }

        /// Creates a mutable reference to a note event of this type from a reference to a raw,
        /// C-FFI compatible event struct.
        ///
        /// # Panics
        ///
        /// This method will panic if the given event struct's header doesn't actually match
        /// the expected note event type.
        #[inline]
        pub const fn from_raw_mut(raw: &mut $raw_type) -> &mut Self {
            crate::events::ensure_event_matches::<Self>(&raw.header);

            // SAFETY: This type is #[repr(C)]-compatible with $raw_type
            unsafe { &mut *(raw as *mut $raw_type as *mut Self) }
        }
    };
}

pub(crate) use impl_event_helpers;
