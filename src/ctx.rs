use git2::Repository;

use crate::error::Maybe;

#[derive(PartialEq)]
pub enum Mode {
    Unknown,
    Cli,
    Background,
    Pipe,
}

pub struct Ctx {
    pub repo: Repository,
    mode: Mode,
    no_color: bool,
}

impl Ctx {
    pub fn set_mode(&mut self, n: Mode) {
        self.mode = n;
    }

    pub fn can_prompt(&self) -> bool {
        match self.mode {
            Mode::Unknown => true,
            Mode::Cli => true,
            Mode::Background => false,
            Mode::Pipe => false,
        }
    }

    pub fn disable_color(&mut self) {
        self.no_color = true;
    }

    pub fn color_enabled(&self) -> bool {
        !self.no_color && self.mode != Mode::Pipe
    }

    pub fn is_pipe(&self) -> bool {
        return self.mode == Mode::Pipe;
    }
}

pub fn init_ctx() -> Maybe<Ctx> {
    let repo = Repository::open_from_env()?;
    return Ok(Ctx {
        repo,
        mode: Mode::Unknown,
        no_color: false,
    });
}
