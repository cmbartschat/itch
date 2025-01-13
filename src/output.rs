use std::{
    env,
    fmt::{self},
};

use crate::error::Maybe;

pub enum OutputTarget {
    Pager(minus::Pager),
    Stdout,
}

impl OutputTarget {
    pub fn new() -> Maybe<Self> {
        if env::var_os("NOPAGER").is_some() {
            Ok(OutputTarget::Stdout)
        } else {
            let p = minus::Pager::new();
            p.horizontal_scroll(true)
                .expect("Failed to communicate with the pager");
            Ok(OutputTarget::Pager(p))
        }
    }

    pub fn finish(self) {
        match self {
            Self::Pager(p) => minus::page_all(p).unwrap(),
            Self::Stdout => {}
        }
    }
}

impl fmt::Write for OutputTarget {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self {
            Self::Pager(p) => p.write_str(s),
            Self::Stdout => Ok(print!("{s}")),
        }
    }
}
