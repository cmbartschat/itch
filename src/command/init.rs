use anyhow::{anyhow, bail};
use gix::{ObjectId, hash::Kind};

use crate::error::Attempt;

pub fn init_command() -> Attempt {
    let path =
        std::env::current_dir().map_err(|_| anyhow!("failed to resolve current directory."))?;

    match gix::open(path.clone()) {
        Ok(_) => {
            bail!("a repository already exists in this location");
        }
        Err(e) => match e {
            gix::open::Error::NotARepository { .. } => {}
            e => Err(e)?,
        },
    };
    let repo = gix::init(path)?;
    repo.new_commit(
        "Initial commit",
        ObjectId::empty_tree(Kind::Sha1),
        Vec::<ObjectId>::new(),
    )?;

    Ok(())
}
