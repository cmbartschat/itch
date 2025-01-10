use std::{
    env,
    fmt::{self},
    io::Stdout,
};

use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};

use crate::error::Maybe;

pub struct PagerState {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    lines: Vec<String>,
}

pub enum OutputTarget {
    Pager(PagerState),
    Stdout,
}

impl OutputTarget {
    pub fn new() -> Maybe<Self> {
        if env::var_os("NOPAGER").is_some() {
            Ok(OutputTarget::Stdout)
        } else {
            let stdout = std::io::stdout();
            let backend = CrosstermBackend::new(stdout);
            let terminal = Terminal::new(backend).expect("Terminal creation");
            Ok(OutputTarget::Pager(PagerState {
                terminal,
                lines: vec![],
            }))
        }
    }

    pub fn finish(self) {
        match self {
            Self::Pager(p) => {
                // p.terminal.clear();
            }
            Self::Stdout => {}
        }
    }
}

impl fmt::Write for OutputTarget {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        match self {
            Self::Pager(ref mut p) => {
                p.lines.push(s.into());

                p.terminal.clear().expect("clear");
                p.terminal
                    .draw(|f| {
                        let size = f.size();
                        let block = Block::default().title("Block");
                        f.render_widget(block, size);
                    })
                    .expect("Redraw");
                Ok(())
            }
            Self::Stdout => Ok(print!("{s}")),
        }
    }
}
