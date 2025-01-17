use crate::{
    cli::SquashArgs,
    ctx::Ctx,
    error::{fail, Attempt},
    save::resolve_commit_message,
};

pub fn squash_command(ctx: &Ctx, args: &SquashArgs) -> Attempt {
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

    let message = match resolve_commit_message(&args.message) {
        Some(m) => m,
        None => match top_commit.message() {
            Some(m) => m.to_string(),
            None => return fail("Invalid characters in previous message"),
        },
    };

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

    Ok(())
}
