use git2::{build::TreeUpdateBuilder, Error};
use log::debug;

use crate::{cli::SaveArgs, ctx::Ctx};

pub fn _save_command(ctx: &Ctx, args: &SaveArgs) -> Result<(), Error> {
    let repo = &ctx.repo;

    let mut builder = TreeUpdateBuilder::new();

    let mut index = ctx.repo.index()?;

    let index_commit = index.write_tree()?;

    let index_tree = repo.find_tree(index_commit)?;

    // let tree_commit = builder.write()?;

    builder.upsert(
        "testing.txt",
        repo.blob(&"Testing".as_bytes())?,
        git2::FileMode::Blob,
    );

    let combined_id = builder.create_updated(&repo, &index_tree)?;

    let combined_tree = repo.find_tree(combined_id)?;

    // ctx.repo.revparse("HEAD^{tree}")?;

    let message = args.message.clone().unwrap_or(String::from("Save"));

    let signature = &ctx.repo.signature()?;

    let parent = &ctx.repo.head()?.peel_to_commit()?;

    let commit = ctx.repo.commit(
        None,
        &signature,
        &signature,
        &message,
        &combined_tree,
        &[&parent],
    )?;

    debug!("Committed: {}", commit);

    Ok(())
}

pub fn save_command(ctx: &Ctx, args: &SaveArgs) -> Result<(), ()> {
    return _save_command(ctx, args).map_err(|_| ());
}
