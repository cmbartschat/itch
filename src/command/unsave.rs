use git2::{Error, ResetType};

use crate::ctx::Ctx;

pub fn unsave_command(ctx: &Ctx) -> Result<(), Error> {
    let head_commit = ctx.repo.head()?.peel_to_commit()?;
    let base_commit = ctx
        .repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;

    let fork_commit = ctx
        .repo
        .find_commit(ctx.repo.merge_base(base_commit.id(), head_commit.id())?)?;

    ctx.repo
        .reset(&fork_commit.into_object(), ResetType::Soft, None)?;

    Ok(())
}
