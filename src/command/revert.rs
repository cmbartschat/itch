use anyhow::bail;

use crate::{
    branch::find_main, cli::RevertArgs, ctx::Ctx, error::Attempt, reset::pop_and_reset,
    save::save_temp,
};

pub fn revert_command(ctx: &Ctx, args: &RevertArgs) -> Attempt {
    save_temp(ctx, "Save before revert".to_string())?;

    let head_commit = ctx.repo.head()?.peel_to_commit()?;
    let base_commit = find_main(ctx)?.peel_to_commit()?;

    let fork_commit = ctx
        .repo
        .find_commit(ctx.repo.merge_base(base_commit.id(), head_commit.id())?)?;

    if args.args.is_empty() {
        bail!("Missing files to revert.");
    }

    todo!();
    // let tree = fork_commit.tree()?;
    // let mut options = CheckoutBuilder::new();

    // options.force();
    // options.remove_untracked(true);

    // for file in &args.args {
    //     options.path(file);
    // }

    // ctx.repo
    //     .checkout_tree(&tree.into_object(), Some(&mut options))?;

    pop_and_reset(ctx)?;

    Ok(())
}
