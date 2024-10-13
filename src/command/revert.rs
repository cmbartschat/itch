use git2::build::CheckoutBuilder;

use crate::{
    cli::RevertArgs,
    ctx::Ctx,
    error::{fail, Attempt},
};

pub fn revert_command(ctx: &Ctx, args: &RevertArgs) -> Attempt {
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
        return fail("Missing files to revert.");
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
    Ok(())
}
