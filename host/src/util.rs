use std::ptr::NonNull;
use std::sync::{Arc, RwLock, Weak};

struct SendPtr<T>(Option<NonNull<T>>);
unsafe impl<T: Send> Send for SendPtr<T> {}
unsafe impl<T: Sync> Sync for SendPtr<T> {}

pub struct WriterLock<T: 'static> {
    // This must *only* be dropped by this thread
    data: Arc<T>,
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
            data,
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
            let ptr = match lock.get_mut() {
                Ok(ptr) => ptr,
                Err(err) => err.into_inner(),
            };

            // We set the pointer to null anyway just in case
            ptr.0 = None;
            return;
        }

        // We're here because some reader did upgrade but did not lock yet. So we try to grab it first.

        // We ignore poisons: we're just letting go of a pointer
        let mut ptr = match self.lock.write() {
            Ok(ptr) => ptr,
            Err(err) => err.into_inner(),
        };

        // We set the pointer to null, so that when we release the lock, the reader won't find a
        // pointer there anymore.
        ptr.0 = None;
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
        // the Option to None through a lock. It can't be invalid if we're here and it's None.
        let data = unsafe { locked.0?.as_ref() };
        let result = Some(lambda(data));

        drop(locked);

        result
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
