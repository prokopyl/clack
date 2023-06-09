use clack_extensions::timer::TimerId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Timers {
    latest_id: u32,
    smallest_duration: Option<u32>,
    timers: HashMap<TimerId, Timer>,
}

impl Timers {
    pub fn new() -> Self {
        Self {
            latest_id: 0,
            timers: HashMap::new(),
            smallest_duration: None,
        }
    }

    pub fn tick_all(&mut self) -> impl Iterator<Item = TimerId> + '_ {
        let now = Instant::now();

        self.timers
            .values_mut()
            .filter_map(move |t| t.tick(now).then_some(t.id))
    }

    pub fn register_new(&mut self, interval: u32) -> TimerId {
        let interval = interval.max(10);

        self.latest_id += 1;
        let id = TimerId(self.latest_id);

        println!("Plugin registered new Timer with ID ({id}), running every {interval}ms.");
        self.timers.insert(id, Timer::new(id, interval));

        match self.smallest_duration {
            None => self.smallest_duration = Some(interval),
            Some(smallest) if smallest > interval => self.smallest_duration = Some(interval),
            _ => {}
        }

        id
    }

    pub fn unregister(&mut self, id: TimerId) -> bool {
        if self.timers.remove(&id).is_some() {
            println!("Unregistered Timer with id {id}.");
            self.smallest_duration = self.timers.values().map(|t| t.interval).min();
            true
        } else {
            false
        }
    }

    pub fn smallest_duration(&self) -> Option<Duration> {
        self.smallest_duration
            .map(|i| Duration::from_millis(i as u64))
    }
}

struct Timer {
    id: TimerId,
    interval: u32,
    last_updated_at: Option<Instant>,
}

impl Timer {
    fn new(id: TimerId, interval: u32) -> Self {
        Self {
            id,
            interval,
            last_updated_at: None,
        }
    }

    fn tick(&mut self, now: Instant) -> bool {
        let triggered = if let Some(last_updated_at) = self.last_updated_at {
            if let Some(since) = now.checked_duration_since(last_updated_at) {
                since > Duration::from_millis(self.interval as u64)
            } else {
                false
            }
        } else {
            true
        };
        self.last_updated_at = Some(now);

        triggered
    }
}
