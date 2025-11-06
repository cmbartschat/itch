use std::fmt::Display;

use gix::{
    Commit,
    bstr::{self, BStr},
    fs::FileSnapshot,
    objs::Find,
    progress::Discard,
    refs::{
        FullName, FullNameRef,
        transaction::{Change, LogChange, PreviousValue},
    },
    worktree::stack::state::attributes::Source,
};

use crate::{ctx::Ctx, error::Attempt};

pub fn set_head_direct<T>(ctx: &Ctx, target: T) -> Attempt
where
    T: Display,
    FullName: From<T>,
{
    let message = format!("Setting {} as head", target);

    ctx.repo.edit_reference(gix::refs::transaction::RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: gix::refs::transaction::RefLog::AndReference,
                force_create_reflog: false,
                message: message.into(),
            },
            expected: PreviousValue::Any,
            new: gix::refs::Target::Symbolic(target.into()),
        },
        name: ctx.repo.head()?.name().into(),
        deref: false,
    })?;

    Ok(())
}

pub fn set_branch_direct<T>(ctx: &Ctx, name: T, message: &str, commit: Commit) -> Attempt
where
    FullName: From<T>,
{
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
        name: name.into(),
        deref: false,
    })?;

    Ok(())
}
