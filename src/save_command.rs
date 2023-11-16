use git2::{Error, IndexAddOption};
use log::debug;

use crate::{cli::SaveArgs, ctx::Ctx};

pub fn _save_command(ctx: &Ctx, args: &SaveArgs) -> Result<(), Error> {
    let repo = &ctx.repo;

    let mut index = repo.index()?;
    index.add_all(["*"], IndexAddOption::all(), None)?;
    let index_commit = index.write_tree()?;

    let tree = repo.find_tree(index_commit)?;

    let mut message = args.message.join(" ");
    if message.len() == 0 {
        message = String::from("Save");
    }

    let signature = repo.signature()?;

    let parent = repo.head()?.peel_to_commit()?;

    let commit = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &[&parent],
    )?;

    debug!("Committed: {}", commit);

    Ok(())
}

pub fn save_command(ctx: &Ctx, args: &SaveArgs) -> Result<(), ()> {
    return _save_command(ctx, args).map_err(|_| ());
}
