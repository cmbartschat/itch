use std::{
    env,
    fmt::{self},
};

use git2::Error;

pub enum OutputTarget {
    Pager(minus::Pager),
    Stdout,
}

impl OutputTarget {
    pub fn new() -> Result<Self, Error> {
        if env::var_os("NOPAGER").is_some() {
            Ok(OutputTarget::Stdout)
        } else {
            Ok(OutputTarget::Pager(minus::Pager::new()))
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
