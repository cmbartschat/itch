use git2::Repository;
use std::io::IsTerminal;

use crate::error::Maybe;

#[derive(PartialEq)]
pub enum Mode {
    Cli,
    Pipe,
}

#[derive(Clone)]
pub struct EnvCtx {
    pub is_pipe: bool,
    pub color_enabled: bool,
    pub can_prompt: bool,
}

impl EnvCtx {
    pub fn from_cli() -> Self {
        let mode = if std::io::stdout().lock().is_terminal() {
            Mode::Cli
        } else {
            Mode::Pipe
        };
        let no_color_set = std::env::var_os("NO_COLOR").is_some_and(|e| !e.is_empty());
        let color_enabled = !no_color_set && mode != Mode::Pipe;
        Self {
            is_pipe: mode == Mode::Pipe,
            color_enabled,
            can_prompt: match mode {
                Mode::Cli => true,
                Mode::Pipe => false,
            },
        }
    }

    pub fn background() -> Self {
        Self {
            is_pipe: false,
            color_enabled: false,
            can_prompt: false,
        }
    }
}

pub struct Ctx {
    pub repo: Repository,
    env: EnvCtx,
}

impl Ctx {
    pub fn can_prompt(&self) -> bool {
        self.env.can_prompt
    }

    pub fn color_enabled(&self) -> bool {
        self.env.color_enabled
    }

    pub fn is_pipe(&self) -> bool {
        self.env.is_pipe
    }

    pub fn env(&self) -> &EnvCtx {
        &self.env
    }
}

pub fn init_ctx(env: EnvCtx) -> Maybe<Ctx> {
    let repo = Repository::open_from_env()?;
    Ok(Ctx { repo, env })
}
