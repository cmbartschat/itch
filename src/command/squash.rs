use git2::{Commit, Oid};

use crate::{
    cli::SquashArgs,
    ctx::Ctx,
    error::{Attempt, Maybe, fail},
    save::{include_footer, resolve_commit_message},
};

pub fn resolve_squashed_message(
    ctx: &Ctx,
    message: &[String],
    top_commit: Commit,
    fork_id: Oid,
) -> Maybe<String> {
    if let Some(m) = resolve_commit_message(message) {
        return include_footer(ctx, &m);
    }

    let mut commit = top_commit;
    while commit.id() != fork_id {
        match commit.summary() {
            None => return fail!("Invalid characters in previous message"),
            Some(m) => {
                if m == "Save" {
                    commit = commit.parent(0)?;
                } else {
                    return Ok(m.to_string());
                }
            }
        }
    }
    include_footer(ctx, "Squash")
}

pub fn squash_command(ctx: &Ctx, args: &SquashArgs) -> Attempt {
    let signature = ctx.repo.signature()?;

    let latest_main = ctx
        .repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;

    let top_commit = ctx.repo.head()?.peel_to_commit()?;

    let fork_id = ctx.repo.merge_base(latest_main.id(), top_commit.id())?;
    if top_commit.id() == fork_id {
        return Ok(());
    }

    let parent = ctx.repo.find_commit(fork_id)?;

    let tree = top_commit.tree()?;

    let message = resolve_squashed_message(ctx, &args.message, top_commit, fork_id)?;

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

    Ok(())
}
