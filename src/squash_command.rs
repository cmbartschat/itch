use git2::Error;
use log::debug;

use crate::ctx::Ctx;

pub fn squash_command(ctx: &Ctx) -> Result<(), Error> {
    debug!("You want me to squash");

    let _head = ctx.repo.head()?;

    let signature = ctx.repo.signature()?;

    let latest_main = ctx
        .repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;

    let top_commit = ctx.repo.head()?.peel_to_commit()?;

    let parent_id = ctx.repo.merge_base(latest_main.id(), top_commit.id())?;

    let parent = ctx.repo.find_commit(parent_id)?;

    if top_commit.parents().any(|f| f.id() == parent.id()) {
        println!("Already squashed.");
        return Ok(());
    }

    let message = top_commit.message().unwrap_or("<invalid message>");
    let tree = top_commit.tree()?;

    let squashed_commit = ctx.repo.find_commit(ctx.repo.commit(
        None,
        &signature,
        &signature,
        &message,
        &tree,
        &[&parent],
    )?)?;

    let squashed_object = squashed_commit.as_object();

    ctx.repo
        .reset(squashed_object, git2::ResetType::Mixed, None)?;

    println!("Squashed to {}", squashed_commit.id());

    Ok(())
}
