use std::any::type_name;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A low-level utility used to implement custom [`Factory`] pointer types.
///
/// This is a pointer type that allows read-only access to the pointed data, except that it does
/// not deal with references or borrows. You cannot get a reference to the factory structure this
/// points to, only [`get`] a copy of the factory structure.
///
/// This allows to more easily avoid violating Rust's strict [reference rules]. Instead, this type
/// only needs the pointer to adhere to the [`ptr::read`] function's safety rules.
///
/// [`Factory`]: crate::factory::Factory
/// [`get`]: Self::get
/// [reference rules]: https://doc.rust-lang.org/std/ptr/index.html#pointer-to-reference-conversion
/// [`ptr::read`]: core::ptr::read
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct RawFactoryPointer<'a, T> {
    inner: NonNull<T>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> RawFactoryPointer<'a, T> {
    /// Creates a new [`RawFactoryPointer`] from a shared reference to the factory data structure.
    ///
    /// The lifetime of this new pointer is tied to the lifetime of the given reference.
    #[inline]
    pub fn new(inner: &'a T) -> Self {
        Self {
            inner: inner.into(),
            _marker: PhantomData,
        }
    }

    /// Creates a new [`RawFactoryPointer`] from a [non-null](NonNull) pointer to the factory data
    /// structure.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated for the duration of
    /// the `'a` lifetime:
    ///
    /// * `raw` must be [valid] for reads;
    /// * `raw` must be properly aligned (even if `T` has size `0`);
    /// * `raw` must point to a properly initialized value of type `T`;
    /// * the memory `raw` points to must not be written to, it can only be read from.
    ///
    /// These requirements are similar to those of the [`ptr::read`] function, except this function
    /// cannot cause UB if `T` is not [`Copy`].
    /// This is because the only method on this type that actually performs a read,
    /// [`get`](Self::get), checks this statically via a [`Copy`] bound.
    ///
    /// [valid]: core::ptr#safety
    /// [`ptr::read`]: core::ptr::read
    #[inline]
    pub const unsafe fn from_raw(raw: NonNull<T>) -> Self {
        Self {
            inner: raw,
            _marker: PhantomData,
        }
    }

    /// Returns a [non-null](NonNull) pointer to the factory structure this pointer points to.
    #[inline]
    pub const fn as_raw(&self) -> NonNull<T> {
        self.inner
    }

    /// Returns a raw pointer to the factory structure this pointer points to.
    #[inline]
    pub const fn as_ptr(&self) -> *mut T {
        self.inner.as_ptr()
    }
}

impl<T: Copy> RawFactoryPointer<'_, T> {
    /// Reads this pointer to return a copy of the factory structure.
    ///
    /// This function is always safe to use.
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

// SAFETY: We are only using this pointer for reads, and the memory it points to should be read-only.
unsafe impl<T: Send> Send for RawFactoryPointer<'_, T> {}
// SAFETY: We are only using this pointer for reads, and the memory it points to should be read-only.
unsafe impl<T: Sync> Sync for RawFactoryPointer<'_, T> {}
