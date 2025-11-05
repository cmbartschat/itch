use gix::Commit;

use crate::{
    consts::{TEMP_COMMIT_PREFIX, TEMP_COMMIT_PREFIX_BYTES},
    ctx::Ctx,
    error::{Attempt, Maybe},
};

pub fn reset_repo(ctx: &Ctx) -> Attempt {
    let object = ctx.repo.head()?.peel_to_commit()?;
    todo!();
    // ctx.repo.reset(&object, ResetType::Mixed, None)?;
    Ok(())
}

fn is_temp_commit(c: &Commit) -> Maybe<bool> {
    Ok(c.parent_ids().count() == 1
        && c.message()?
            .body()
            .is_some_and(|m| m.starts_with(TEMP_COMMIT_PREFIX_BYTES)))
}

pub fn pop_and_reset(ctx: &Ctx) -> Attempt {
    let mut commit = ctx.repo.head()?.peel_to_commit()?;

    while is_temp_commit(&commit)? {
        let parent_commit = commit
            .parent_ids()
            .next()
            .unwrap()
            .try_object()?
            .unwrap()
            .try_into_commit()?;

        commit = parent_commit;
    }

    todo!();
    // ctx.repo
    //     .reset(&commit.into_object(), ResetType::Mixed, None)?;

    Ok(())
}
