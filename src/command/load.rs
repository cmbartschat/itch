use gix::{bstr::Find, worktree::stack::state::attributes::Source};

use crate::{
    branch::find_branch, cli::LoadArgs, ctx::Ctx, error::Attempt, reset::pop_and_reset,
    save::save_temp,
};

fn load_command_inner(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    let target_ref = find_branch(ctx, &args.name)?;

    let mut options = ctx.repo.checkout_options(Source::IdMapping)?;
    options.overwrite_existing = true;

    todo!();
    // gix();
    // gix::worktree::state::checkout(
    //     ctx.repo.worktree().unwrap().index().unwrap(),
    //     ctx.repo.workdir().unwrap(),
    //     ctx.repo.workdir().unwrap().f,
    //     None,
    //     None,
    //     false,
    //     options,
    // );

    pop_and_reset(ctx)
}

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    save_temp(ctx, format!("Save before switching to {}", args.name))?;

    load_command_inner(ctx, args)
}
