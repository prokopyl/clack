use std::cell::UnsafeCell;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, Weak};

struct SendPtr<T>(Option<NonNull<T>>);
// SAFETY: implemented only if T is Send
unsafe impl<T: Send> Send for SendPtr<T> {}
// SAFETY: implemented only if T is Sync
unsafe impl<T: Sync> Sync for SendPtr<T> {}

pub struct WriterLock<T: 'static> {
    // This must *only* be dropped by this thread
    data: ManuallyDrop<Arc<T>>,
    // It's okay if other readers drop this
    lock: Arc<RwLock<SendPtr<T>>>,
}

#[cold]
fn panic_poisoned() -> ! {
    panic!("PluginInstance panicked while (de)activating and is poisoned")
}

impl<T: 'static> WriterLock<T> {
    pub fn new(data: Arc<T>) -> Self {
        // SAFETY: The pointer comes from the Arc, it has to be non-null.
        let ptr = unsafe { NonNull::new_unchecked(Arc::as_ptr(&data) as *mut _) };

        Self {
            data: ManuallyDrop::new(data),
            lock: Arc::new(RwLock::new(SendPtr(Some(ptr)))),
        }
    }

    pub fn make_reader(&self) -> WeakReader<T> {
        WeakReader {
            lock: Arc::downgrade(&self.lock),
        }
    }

    #[inline]
    pub fn get(&self) -> &Arc<T> {
        // SAFETY: We're the only writer, and can only write through &mut.
        // Others may be reading, but this doesn't prevent us from reading as well.
        &self.data
    }

    pub fn use_mut<TR>(&mut self, lambda: impl FnOnce(&mut Arc<T>) -> TR) -> TR {
        // "Faster path" if no weak readers exists
        let result = if Arc::get_mut(&mut self.lock).is_some() {
            lambda(&mut self.data)
        } else {
            let Ok(lock) = self.lock.write() else {
                panic_poisoned()
            };
            let result = lambda(&mut self.data);
            drop(lock);
            result
        };

        result
    }
}

impl<T: 'static> Drop for WriterLock<T> {
    fn drop(&mut self) {
        if let Some(lock) = Arc::get_mut(&mut self.lock) {
            // We don't have any readers. It's an easy drop!

            // We ignore poisons: we're just letting go of a pointer
            let ptr = lock.get_mut().unwrap_or_else(|err| err.into_inner());

            // We set the pointer to null anyway just in case
            ptr.0 = None;
        } else {
            // We're here because a weak reader exists.

            // Some reader could have upgraded but did not lock yet.
            // If we grab the lock first, the reader will find a None pointer.
            // If we try to grab the lock too late, it will block until the reader is done.

            // We ignore poisons: we're just letting go of a pointer
            let mut ptr = self.lock.write().unwrap_or_else(|err| err.into_inner());

            // We set the pointer to null, so that when we release the lock, the reader won't find a
            // pointer there anymore.
            ptr.0 = None;
        }

        // From this point on, we are now guaranteed that no weak reader can access our copy of
        // the data Arc.

        // In either case, the Arc could have been externally cloned.
        // So we only drop the inner value if we are the sole owner.
        if Arc::get_mut(&mut self.data).is_some() {
            // SAFETY: We can only call this once (as we're in Drop), and we never use the inner
            // value again afterward.
            unsafe { ManuallyDrop::drop(&mut self.data) }
        };
    }
}

pub struct WeakReader<T> {
    lock: Weak<RwLock<SendPtr<T>>>,
}

impl<T> WeakReader<T> {
    pub fn use_with<TR>(&self, lambda: impl FnOnce(&T) -> TR) -> Option<TR> {
        let inner_lock = self.lock.upgrade()?;
        let Ok(locked) = inner_lock.read() else {
            panic_poisoned()
        };

        // SAFETY: The only way the pointer is invalidated is during the writer's drop, which sets
        // the Option to None through the lock. It can't be invalid if we're here while it's not None.
        let data = unsafe { locked.0?.as_ref() };
        let result = lambda(data);

        drop(locked);

        Some(result)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    pub fn works_with_simultaneous_drop_and_access() {
        let mut writer = WriterLock::new(Arc::new(false));

        let reader = writer.make_reader();

        // EVIL!!!
        let upgraded = reader.lock.upgrade().unwrap();

        // Write to it
        writer.use_mut(|b| *Arc::get_mut(b).unwrap() = true);

        let evil_thread = thread::spawn(move || {
            {
                let lock = upgraded.read().unwrap();
                thread::sleep(Duration::from_millis(300));

                // Read it, it should still be valid
                // SAFETY: the data is guaranteed to still be alive at this time
                let data = unsafe { lock.0.unwrap().as_ref() };
                assert!(*data);
            }

            // Wait for drop to do its work
            thread::sleep(Duration::from_millis(200));

            assert_eq!(reader.use_with(|d| *d), None);
        });

        thread::sleep(Duration::from_millis(100));

        // Drop. This should lock until the other thread has locked it itself.
        drop(writer);

        evil_thread.join().unwrap();
    }
}
