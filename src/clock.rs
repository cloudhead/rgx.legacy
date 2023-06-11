use std::{thread, time};

pub struct Clock {
    last: time::Instant,
}

impl Clock {
    /// Create a new clock.
    pub fn new(last: time::Instant) -> Self {
        Self { last }
    }

    /// Tick at a target number of frames per second.
    pub fn tick(&mut self, fps: f64) -> time::Duration {
        let target = time::Duration::from_secs_f64(1. / fps);
        let delta = self.last.elapsed();

        if delta < target {
            thread::sleep(target - delta);
        }
        let now = time::Instant::now();
        let delta = now - self.last;

        self.last = now;

        delta
    }
}
