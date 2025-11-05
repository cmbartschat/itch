use gix::{
    Commit,
    index::write,
    refs::transaction::{Change, LogChange, PreviousValue},
};

use crate::{
    consts::{TEMP_COMMIT_PREFIX, TEMP_COMMIT_PREFIX_BYTES},
    ctx::Ctx,
    error::{Attempt, Maybe},
};

pub fn reset_repo(ctx: &Ctx) -> Attempt {
    let object = ctx.repo.head()?.peel_to_commit()?;
    ctx.repo
        .index_from_tree(&object.tree()?.id)?
        .write(Default::default())?;

    Ok(())
}

fn is_temp_commit(c: &Commit) -> Maybe<bool> {
    let parent_count = c.parent_ids().count();
    if parent_count != 1 {
        Ok(false)
    } else {
        let has_temp_message = c.message()?.summary().starts_with(TEMP_COMMIT_PREFIX_BYTES);

        Ok(has_temp_message)
    }
}

pub fn pop_and_reset(ctx: &Ctx) -> Attempt {
    eprintln!("Popping and resetting");
    let mut current_commit = ctx.repo.head()?.peel_to_commit()?;
    let initial_id = current_commit.id;
    let mut commit = current_commit;

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

    let message = format!("Popping temporary commit");

    if commit.id() != initial_id {
        eprintln!("Temporary commit needs to be popped, {commit:?} vs {initial_id:?}");
        ctx.repo.edit_reference(gix::refs::transaction::RefEdit {
            change: Change::Update {
                log: LogChange {
                    mode: gix::refs::transaction::RefLog::AndReference,
                    force_create_reflog: false,
                    message: message.into(),
                },
                expected: PreviousValue::Any,
                new: gix::refs::Target::Object(commit.id),
            },
            name: ctx.repo.head()?.name().into(),
            deref: true,
        })?;
    }

    // let mut index = ctx.repo.index_from_tree(&commit.tree()?.id)?;
    // let mut write_options = write::Options::default();
    // index.write(write_options)?;

    Ok(())
}
