use std::time::{Duration, Instant};

pub struct SlideshowController {
    enabled: bool,
    interval: Duration,
    last_tick: Instant,
}

impl SlideshowController {
    pub fn new() -> Self {
        Self {
            enabled: false,
            interval: Duration::from_secs(3),
            last_tick: Instant::now(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.last_tick = Instant::now();
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_interval_sec(&mut self, sec: u32) {
        self.interval = Duration::from_secs(sec as u64);
    }

    pub fn interval_sec(&self) -> u32 {
        self.interval.as_secs() as u32
    }

    pub fn reset_tick(&mut self) {
        self.last_tick = Instant::now();
    }

    pub fn should_advance(&self) -> bool {
        if !self.enabled {
            return false;
        }
        self.last_tick.elapsed() >= self.interval
    }
}
