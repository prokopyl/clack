use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::ptr;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};

/// Equivalent in spirit to `UnsafeCell<Option<T>>`, except you can read if the cell is set or not
/// without invalidating potential active &mut references to the data.
pub(crate) struct UnsafeOptionCell<T> {
    is_some: AtomicBool,
    inner: UnsafeCell<MaybeUninit<T>>,
}

impl<T> UnsafeOptionCell<T> {
    pub(crate) fn new() -> Self {
        Self {
            is_some: AtomicBool::new(false),
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn is_some(&self) -> bool {
        self.is_some.load(Ordering::Relaxed)
    }

    pub fn as_ptr(&self) -> Option<NonNull<T>> {
        if !self.is_some() {
            return None;
        }

        let ptr = self.inner.get().cast();

        // SAFETY: this pointer comes from an UnsafeCell, it cannot be null.
        unsafe { Some(NonNull::new_unchecked(ptr)) }
    }

    /// # Safety
    /// Users must ensure the option is initialized to a value.
    pub unsafe fn as_ptr_unchecked(&self) -> NonNull<T> {
        let ptr = self.inner.get().cast();

        // SAFETY: this pointer comes from an UnsafeCell, it cannot be null.
        unsafe { NonNull::new_unchecked(ptr) }
    }

    /// # Safety
    /// Users must ensure this method is never called concurrently with itself, [`Self::take`], or
    /// while any reference to `T` is still being held.
    pub unsafe fn put(&self, value: T) {
        if let Err(true) =
            self.is_some
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        {
            // Drop the old value if there was one already.
            ptr::drop_in_place(self.inner.get().cast::<T>())
        }

        self.inner.get().write(MaybeUninit::new(value));
    }

    /// # Safety
    /// Users must ensure this method is never called concurrently with itself, [`Self::put`], or
    /// while any reference to `T` is still being held.
    pub unsafe fn take(&self) -> Option<T> {
        if let Ok(true) =
            self.is_some
                .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        {
            Some(self.inner.get().cast::<T>().read())
        } else {
            None
        }
    }
}

impl<T> Drop for UnsafeOptionCell<T> {
    fn drop(&mut self) {
        let is_some = self.is_some.get_mut();
        if *is_some {
            // SAFETY: is_some guarantees that the data is in an initialized state
            unsafe { self.inner.get_mut().assume_init_drop() }
        }
        *is_some = false;
    }
}

#[inline]
pub(crate) const fn check_collection_clap_size_overflow<T>(value: &[T]) {
    #[cold]
    #[inline(never)]
    const fn too_big() -> ! {
        panic!("CLAP size (u32) overflowed")
    }

    if value.len() >= u32::MAX as usize {
        too_big()
    }
}
