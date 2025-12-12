use std::any::type_name;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct RawFactoryPointer<'a, T> {
    inner: NonNull<T>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> RawFactoryPointer<'a, T> {
    #[inline]
    pub fn new(inner: &'a T) -> Self {
        Self {
            inner: inner.into(),
            _marker: PhantomData,
        }
    }

    /// # Safety
    ///
    /// The object the pointer points to must remain valid for reads for the duration of the `'a` lifetime.
    #[inline]
    pub const unsafe fn from_raw(inner: NonNull<T>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub const fn as_raw(&self) -> NonNull<T> {
        self.inner
    }
}

impl<T: Copy> RawFactoryPointer<'_, T> {
    #[inline]
    pub fn get(&self) -> T {
        // SAFETY: T is Copy, and this type guarantees the pointer is valid for reads for 'a
        unsafe { self.inner.read() }
    }
}

impl<T> Debug for RawFactoryPointer<'_, T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:p})", type_name::<T>(), self.inner)
    }
}

// SAFETY: We are only using this pointer read-only
unsafe impl<T: Send> Send for RawFactoryPointer<'_, T> {}
// SAFETY: We are only using this pointer read-only TODO
unsafe impl<T: Sync> Sync for RawFactoryPointer<'_, T> {}
