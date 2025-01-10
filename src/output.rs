use std::{
    env,
    fmt::{self},
};

use pager_rs::{CommandList, State, StatusBar};

use crate::error::Maybe;

pub enum OutputTarget<'a> {
    Pager(State<'a>),
    Stdout,
}

impl OutputTarget<'_> {
    pub fn new() -> Maybe<Self> {
        if env::var_os("NOPAGER").is_some() {
            Ok(OutputTarget::Stdout)
        } else {
            let status_bar = StatusBar::new("".to_string());
            let mut state =
                State::<'_>::new("test".to_string(), status_bar, CommandList::default())
                    .expect("Could not open output stream.");

            state.show_line_numbers = false;

            pager_rs::init().expect("Pager init");
            pager_rs::run(&mut state).expect("Pager start");
            Ok(OutputTarget::Pager(state))
        }
    }

    pub fn finish(self) {
        match self {
            Self::Pager(p) => {
                pager_rs::finish().expect("Finish");
            }
            Self::Stdout => {}
        }
    }
}

impl fmt::Write for OutputTarget<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self {
            Self::Pager(p) => {
                p.content = format!("{}\n{}", p.content, s);
                p.home();
                Ok(())
            }
            Self::Stdout => Ok(print!("{s}")),
        }
    }
}
