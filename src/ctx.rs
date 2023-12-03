use git2::Repository;

pub struct Ctx {
    pub repo: Repository,
}

pub fn init_ctx() -> Result<Ctx, ()> {
    let repo = Repository::open_from_env().map_err(|e| {
        println!("Could not load repo: {}", e.to_string());
    })?;

    return Ok(Ctx { repo });
}
