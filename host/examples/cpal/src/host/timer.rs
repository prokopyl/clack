use crate::host::CpalHostMainThread;
use clack_extensions::timer::{HostTimerImpl, TimerError, TimerId};
use std::collections::HashMap;
use std::time::{Duration, Instant};

impl<'a> HostTimerImpl for CpalHostMainThread<'a> {
    fn register_timer(&mut self, period_ms: u32) -> Result<TimerId, TimerError> {
        Ok(self
            .timers
            .register_new(Duration::from_millis(period_ms as u64)))
    }

    fn unregister_timer(&mut self, timer_id: TimerId) -> Result<(), TimerError> {
        if self.timers.unregister(timer_id) {
            Ok(())
        } else {
            Err(TimerError::UnregisterError)
        }
    }
}

impl<'a> CpalHostMainThread<'a> {
    /// Ticks all of the registered timers, and run the plugin's callback for all timers that were
    /// triggered.
    pub fn tick_timers(&mut self) {
        let Some(timer) = self.timer_support else { return };
        let plugin = self.plugin.as_mut().unwrap();

        for triggered in self.timers.tick_all() {
            timer.on_timer(plugin, triggered);
        }
    }
}

/// Handles all Timer logic.
pub struct Timers {
    /// All the registered timers, indexed by ID.
    timers: HashMap<TimerId, Timer>,
    /// The last timer ID that was issued.
    latest_id: u32,
    /// The smallest timer duration that was registered.
    /// Useful for configuring the shortest timeout in the event loop without being excessive.
    smallest_duration: Option<Duration>,
}

impl Timers {
    /// Initializes timer logic.
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            latest_id: 0,
            smallest_duration: None,
        }
    }

    /// Ticks all of the registered timer, returning an iterator of those that have just been
    /// triggered.
    pub fn tick_all(&mut self) -> impl Iterator<Item = TimerId> + '_ {
        let now = Instant::now();

        self.timers
            .values_mut()
            .filter_map(move |t| t.tick(now).then_some(t.id))
    }

    /// Registers a new timer that will trigger at a given interval (in ms). Returns the newly
    /// created timer's unique ID.
    pub fn register_new(&mut self, interval: Duration) -> TimerId {
        /// The maximum interval that can be set by plugin.
        /// The spec recommends 30ms at most, we use 10ms to allow for smoth UI updates.
        const MAX_INTERVAL: Duration = Duration::from_millis(10);
        let interval = interval.max(MAX_INTERVAL);

        self.latest_id += 1;
        let id = TimerId(self.latest_id);

        println!(
            "Plugin registered new Timer with ID ({id}), running every {}ms.",
            interval.as_millis()
        );
        self.timers.insert(id, Timer::new(id, interval));

        match self.smallest_duration {
            None => self.smallest_duration = Some(interval),
            Some(smallest) if smallest > interval => self.smallest_duration = Some(interval),
            _ => {}
        }

        id
    }

    /// Unregisters a given timer, specified by its given ID.
    ///
    /// Returns `true` if there was a timer with the given ID, `false` otherwise.
    pub fn unregister(&mut self, id: TimerId) -> bool {
        if self.timers.remove(&id).is_some() {
            println!("Plugin unregistered Timer with ID ({id}).");
            self.smallest_duration = self.timers.values().map(|t| t.interval).min();
            true
        } else {
            false
        }
    }

    /// Gets the smallest duration of all registered timers.
    pub fn smallest_duration(&self) -> Option<Duration> {
        self.smallest_duration
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
