#![allow(dead_code)]
use std::time::Instant;

struct LastStep(String, Instant);

pub struct Timer {
    name: &'static str,
    start: Instant,
    last: Option<LastStep>,
    done: bool,
}

impl Timer {
    pub fn new(name: &'static str) -> Self {
        eprintln!("[{}]: begin", name);
        Self {
            name,
            start: Instant::now(),
            last: None,
            done: false,
        }
    }

    pub fn step(&mut self, name: &'static str) {
        let now = Instant::now();
        let prev_time: &Instant = self.last.as_ref().map(|f| &f.1).unwrap_or(&self.start);
        let passed = now.saturating_duration_since(*prev_time).as_millis();
        let passed_since_start = now.saturating_duration_since(self.start).as_millis();
        eprintln!(
            "[{}]: {} {} (+{})",
            self.name, name, passed_since_start, passed,
        );
        self.last = Some(LastStep(name.into(), now));
    }

    pub fn done(&mut self) {
        if self.done {
            panic!("Can't call done() twice on a timer.");
        }
        let now = Instant::now();
        let prev_time: &Instant = self.last.as_ref().map(|f| &f.1).unwrap_or(&self.start);
        let passed = now.saturating_duration_since(*prev_time).as_millis();
        let passed_since_start = now.saturating_duration_since(self.start).as_millis();
        eprintln!(
            "[{}]: finish {} (+{})",
            self.name, passed_since_start, passed,
        );
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if !self.done {
            self.done();
        }
    }
}
