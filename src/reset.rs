use git2::{Commit, Error, ResetType};

use crate::{consts::TEMP_COMMIT_PREFIX, ctx::Ctx};

pub fn reset_repo(ctx: &Ctx) -> Result<(), Error> {
    let object = ctx.repo.head()?.peel_to_commit()?.into_object();
    ctx.repo.reset(&object, ResetType::Mixed, None)?;
    Ok(())
}

fn is_temp_commit(c: &Commit) -> bool {
    c.parent_count() == 1
        && c.message()
            .map_or(false, |m| m.starts_with(TEMP_COMMIT_PREFIX))
}

pub fn pop_and_reset(ctx: &Ctx) -> Result<(), Error> {
    let mut commit = ctx.repo.head()?.peel_to_commit()?;

    while is_temp_commit(&commit) {
        commit = commit.parents().next().unwrap();
    }

    ctx.repo
        .reset(&commit.into_object(), ResetType::Mixed, None)?;

    Ok(())
}
