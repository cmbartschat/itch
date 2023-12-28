use git2::{Error, Repository};

pub struct Ctx {
    pub repo: Repository,
}

pub fn init_ctx() -> Result<Ctx, Error> {
    let repo = Repository::open_from_env()?;
    return Ok(Ctx { repo });
}
