use git2::{Commit, ResetType};

use crate::{consts::TEMP_COMMIT_PREFIX, ctx::Ctx, error::Attempt};

pub fn reset_repo(ctx: &Ctx) -> Attempt {
    let object = ctx.repo.head()?.peel_to_commit()?.into_object();
    ctx.repo.reset(&object, ResetType::Mixed, None)?;
    Ok(())
}

fn is_temp_commit(c: &Commit) -> bool {
    c.parent_count() == 1
        && c.message()
            .map_or(false, |m| m.starts_with(TEMP_COMMIT_PREFIX))
}

pub fn pop_and_reset(ctx: &Ctx) -> Attempt {
    let mut commit = ctx.repo.head()?.peel_to_commit()?;

    while is_temp_commit(&commit) {
        commit = commit.parents().next().unwrap();
    }

    ctx.repo
        .reset(&commit.into_object(), ResetType::Mixed, None)?;

    Ok(())
}
