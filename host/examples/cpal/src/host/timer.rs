use crate::host::CpalHostMainThread;
use clack_extensions::timer::{HostTimerImpl, PluginTimer, TimerId};
use clack_host::prelude::{HostError, PluginMainThreadHandle};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::time::{Duration, Instant};

impl HostTimerImpl for CpalHostMainThread<'_> {
    fn register_timer(&mut self, period_ms: u32) -> Result<TimerId, HostError> {
        Ok(self
            .timers
            .register_new(Duration::from_millis(period_ms as u64)))
    }

    fn unregister_timer(&mut self, timer_id: TimerId) -> Result<(), HostError> {
        if self.timers.unregister(timer_id) {
            Ok(())
        } else {
            Err(HostError::Message("Unknown timer ID"))
        }
    }
}

/// Handles all Timer logic.
pub struct Timers {
    /// All the registered timers, indexed by ID.
    timers: RefCell<HashMap<TimerId, Timer>>,
    /// The last timer ID that was issued.
    latest_id: Cell<u32>,
    /// The smallest timer duration that was registered.
    /// Useful for configuring the shortest timeout in the event loop without being excessive.
    smallest_duration: Cell<Option<Duration>>,
}

impl Timers {
    /// Initializes timer logic.
    pub fn new() -> Self {
        Self {
            timers: RefCell::new(HashMap::new()),
            latest_id: Cell::new(0),
            smallest_duration: Cell::new(None),
        }
    }

    /// Ticks all the registered timers, returning a vector of those that have just been
    /// triggered.
    fn tick_all(&self) -> Vec<TimerId> {
        // PANIC: This method is not reentrant with any that may also borrow timers
        let mut timers = self.timers.borrow_mut();

        let now = Instant::now();

        timers
            .values_mut()
            .filter_map(move |t| t.tick(now).then_some(t.id))
            .collect()
    }

    /// Ticks all the registered timers, and run the plugin's callback for all timers that were
    /// triggered.
    pub fn tick_timers(&self, timer_ext: &PluginTimer, plugin: &mut PluginMainThreadHandle) {
        for triggered in self.tick_all() {
            timer_ext.on_timer(plugin, triggered);
        }
    }

    /// Registers a new timer that will trigger at a given interval (in ms). Returns the newly
    /// created timer's unique ID.
    pub fn register_new(&self, interval: Duration) -> TimerId {
        /// The maximum interval that can be set by plugin.
        /// The spec recommends 30ms at most, we use 10ms to allow for smoth UI updates.
        const MAX_INTERVAL: Duration = Duration::from_millis(10);
        let interval = interval.max(MAX_INTERVAL);

        let latest_id = self.latest_id.get() + 1;
        self.latest_id.set(latest_id);
        let id = TimerId(latest_id);

        println!(
            "Plugin registered new Timer with ID ({id}), running every {}ms.",
            interval.as_millis()
        );

        // PANIC: This method is not reentrant with any that may also borrow timers
        self.timers
            .borrow_mut()
            .insert(id, Timer::new(id, interval));

        match self.smallest_duration.get() {
            None => self.smallest_duration.set(Some(interval)),
            Some(smallest) if smallest > interval => self.smallest_duration.set(Some(interval)),
            _ => {}
        }

        id
    }

    /// Unregisters a given timer, specified by its given ID.
    ///
    /// Returns `true` if there was a timer with the given ID, `false` otherwise.
    pub fn unregister(&self, id: TimerId) -> bool {
        // PANIC: This method is not reentrant with any that may also borrow timers
        let mut timers = self.timers.borrow_mut();
        if timers.remove(&id).is_some() {
            println!("Plugin unregistered Timer with ID ({id}).");
            self.smallest_duration
                .set(timers.values().map(|t| t.interval).min());
            true
        } else {
            false
        }
    }

    /// Gets the smallest duration of all registered timers.
    pub fn smallest_duration(&self) -> Option<Duration> {
        self.smallest_duration.get()
    }
}

/// A single timer.
struct Timer {
    /// The timer's ID
    id: TimerId,
    /// How often the timer needs to trigger.
    interval: Duration,
    /// The last time the timer was triggered.
    last_triggered_at: Option<Instant>,
}

impl Timer {
    /// Creates a new Timer from its ID and interval.
    fn new(id: TimerId, interval: Duration) -> Self {
        Self {
            id,
            interval,
            last_triggered_at: None,
        }
    }

    /// Ticks this timer. If the time since it was triggered is greater than its configured interval,
    /// it is triggered again and this method returns `true`. Otherwise, nothing happens and this
    /// method returns false.
    ///
    /// This method always returns true if the timer has never been triggered.
    fn tick(&mut self, now: Instant) -> bool {
        let triggered = if let Some(last_updated_at) = self.last_triggered_at {
            if let Some(since) = now.checked_duration_since(last_updated_at) {
                since > self.interval
            } else {
                false
            }
        } else {
            true
        };

        if triggered {
            self.last_triggered_at = Some(now);
        }

        triggered
    }
}
