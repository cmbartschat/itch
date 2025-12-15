use git2::build::CheckoutBuilder;

use crate::{
    cli::RevertArgs,
    ctx::Ctx,
    error::{Attempt, fail},
    reset::pop_and_reset,
    save::save_temp,
};

pub fn revert_command(ctx: &Ctx, args: &RevertArgs) -> Attempt {
    save_temp(ctx, "Save before revert".to_string())?;

    let head_commit = ctx.repo.head()?.peel_to_commit()?;
    let base_commit = ctx
        .repo
        .find_branch("main", git2::BranchType::Local)?
        .into_reference()
        .peel_to_commit()?;

    let fork_commit = ctx
        .repo
        .find_commit(ctx.repo.merge_base(base_commit.id(), head_commit.id())?)?;

    if args.args.is_empty() {
        return fail!("Missing files to revert.");
    }
    let tree = fork_commit.tree()?;
    let mut options = CheckoutBuilder::new();

    options.force();
    options.remove_untracked(true);

    for file in &args.args {
        options.path(file);
    }
    ctx.repo
        .checkout_tree(&tree.into_object(), Some(&mut options))?;

    pop_and_reset(ctx)?;

    Ok(())
}
