use git2::{Error, Repository};

pub enum Mode {
    Unknown,
    Cli,
    Background,
}

pub struct Ctx {
    pub repo: Repository,
    mode: Mode,
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
        }
    }
}

pub fn init_ctx() -> Result<Ctx, Error> {
    let repo = Repository::open_from_env()?;
    return Ok(Ctx {
        repo,
        mode: Mode::Unknown,
    });
}
