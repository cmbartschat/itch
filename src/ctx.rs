use std::path::PathBuf;

use git2::Repository;

pub struct Ctx {
    pub cwd: PathBuf,
    pub repo: Repository,
}

pub fn init_ctx() -> Result<Ctx, ()> {
    let cwd = std::env::current_dir().map_err(|_| ())?;
    let repo = Repository::open(cwd.clone()).map_err(|_| ())?;

    return Ok(Ctx { cwd, repo });
}
