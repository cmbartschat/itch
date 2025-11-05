use crate::{
    branch::find_main, cli::SquashArgs, ctx::Ctx, error::Attempt, save::resolve_commit_message,
};

pub fn squash_command(ctx: &Ctx, args: &SquashArgs) -> Attempt {
    let _head = ctx.repo.head()?;

    let latest_main = find_main(ctx)?.peel_to_commit()?;

    let top_commit = ctx.repo.head()?.peel_to_commit()?;

    let parent_id = ctx.repo.merge_base(latest_main.id(), top_commit.id())?;

    let parent = ctx.repo.find_commit(parent_id)?;

    let message = match resolve_commit_message(&args.message) {
        Some(m) => m,
        None => top_commit
            .message()?
            .map(|f| f.to_string())
            .unwrap_or_default(),
    };

    let tree = top_commit.tree()?;

    let squashed_commit = ctx.repo.new_commit(message, tree.id(), vec![parent.id()])?;

    todo!();
    // ctx.repo
    //     .reset(squashed_object, git2::ResetType::Mixed, None)?;

    Ok(())
}
