use std::time::Instant;

use log::debug;

struct LastStep(String, Instant);

pub struct Timer {
    name: &'static str,
    start: Instant,
    last: Option<LastStep>,
    done: bool,
}

impl Timer {
    pub fn new<'a>(name: &'static str) -> Self {
        return Self {
            name,
            start: Instant::now(),
            last: None,
            done: false,
        };
    }

    pub fn step(&mut self, name: &'static str) {
        let now = Instant::now();
        if let Some(LastStep(prev_name, prev_time)) = &self.last {
            let passed = now.saturating_duration_since(*prev_time);
            debug!(
                "[{}]: {} -> {} {}",
                self.name,
                prev_name,
                name,
                passed.as_millis()
            );
        } else {
            let passed = now.saturating_duration_since(self.start);
            debug!(
                "[{}]: Start -> {}]: {}",
                self.name,
                name,
                passed.as_millis()
            );
        }

        self.last = Some(LastStep(name.into(), now));
    }

    pub fn done(&mut self) {
        if self.done {
            panic!("Can't call done() on the same timer twice.");
        }
        let now = Instant::now();
        let passed = now.saturating_duration_since(self.start);
        debug!("[{}]: Finished after {}", self.name, passed.as_millis())
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if !self.done {
            self.done();
        }
    }
}
