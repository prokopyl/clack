use std::mem::ManuallyDrop;
use std::sync::{Arc, RwLock, RwLockReadGuard, Weak};

pub struct WriterLock<T: 'static> {
    owned: ManuallyDrop<Arc<RwLock<T>>>,
    read_lock: Option<RwLockReadGuard<'static, T>>,
}

#[cold]
fn panic_poisoned() -> ! {
    panic!("PluginInstance panicked while (de)activating and is poisoned")
}

impl<T: 'static> WriterLock<T> {
    pub fn new(inner: T) -> Self {
        let owned = Arc::new(RwLock::new(inner));
        // PANIC: we *just* created the lock. There's no way it can be poisoned already.
        let read_lock: RwLockReadGuard<T> = owned.read().unwrap();

        let read_lock: RwLockReadGuard<'static, T> = unsafe { core::mem::transmute(read_lock) };

        Self {
            owned: ManuallyDrop::new(owned),
            read_lock: Some(read_lock),
        }
    }

    pub fn make_reader(&self) -> WeakReader<T> {
        WeakReader(Arc::downgrade(&self.owned))
    }

    pub fn get(&self) -> &T {
        match &self.read_lock {
            Some(inner) => inner,
            _ => panic_poisoned(),
        }
    }

    pub fn use_mut<TR>(&mut self, lambda: impl FnOnce(&mut T) -> TR) -> TR {
        self.read_lock = None;

        // "Faster path" if no weak readers exists
        let result = if let Some(unshared) = Arc::get_mut(&mut self.owned) {
            let Ok(inner_mut) = unshared.get_mut() else {
                panic_poisoned()
            };
            lambda(inner_mut)
        } else {
            let Ok(mut lock) = self.owned.write() else {
                panic_poisoned()
            };
            lambda(&mut lock)
        };

        let Ok(new_read_lock) = self.owned.read() else {
            unreachable!()
        };

        let new_read_lock: RwLockReadGuard<'static, T> =
            unsafe { core::mem::transmute(new_read_lock) };
        self.read_lock = Some(new_read_lock);

        result
    }
}

impl<T: 'static> Drop for WriterLock<T> {
    fn drop(&mut self) {
        // Drop our own read lock
        self.read_lock = None;

        if Arc::get_mut(&mut self.owned).is_some() {
            // We don't have any readers. It's an easy drop!
            // SAFETY: we don't use the value at all after this.
            unsafe { ManuallyDrop::drop(&mut self.owned) };
            return;
        }

        // There is nothing to drop besides the Arc. We can extract the Arc to drop it and forget the rest.
        let owned = unsafe { ManuallyDrop::take(&mut self.owned) };

        // We're here because some reader did upgrade but did not lock yet. So we wait for it.
        todo!()
    }
}

pub struct WeakReader<T>(Weak<RwLock<T>>);

impl<T> WeakReader<T> {
    pub fn use_with<TR>(&self, lambda: impl FnOnce(&T) -> TR) -> Option<TR> {
        let inner_lock = self.0.upgrade()?;
        let Ok(locked) = inner_lock.read() else {
            panic_poisoned()
        };

        Some(lambda(&locked))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn evil_reader(reader: &WeakReader<()>) {
        let inner_lock = reader.0.upgrade()?;

        barrier.wait(); // EVIL!!

        let Ok(locked) = inner_lock.read() else {
            panic_poisoned()
        };

        drop(locked)
    }

    #[test]
    pub fn works_with_simultaneous_drop_and_access() {
        let writer = WriterLock::new(false);

        let reader = writer.make_reader();

        // EVIL!!!
        let upgraded = reader.0.upgrade().unwrap();

        // Write to it
        writer.use_mut(|b| *b = true);

        let evil_thread = thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            let lock = upgraded.read().unwrap();
            assert_eq!(&*lock, true);
        });

        // Drop. This should lock until the other thread has locked it itself.
        drop(writer);

        evil_thread.join().unwrap();
    }
}
