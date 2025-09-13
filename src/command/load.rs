use git2::build::CheckoutBuilder;

use crate::{
    cli::LoadArgs,
    ctx::Ctx,
    error::{Attempt, fail},
    reset::pop_and_reset,
    save::save_temp,
};

fn load_command_inner(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    let target_ref = ctx
        .repo
        .find_branch(&args.name, git2::BranchType::Local)?
        .into_reference();

    match target_ref.name() {
        Some(name) => ctx.repo.set_head(name)?,
        None => return fail("Invalid branch name"),
    }

    let mut options = CheckoutBuilder::new();
    options.force();
    ctx.repo.checkout_head(Some(&mut options))?;

    pop_and_reset(ctx)
}

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    save_temp(ctx, format!("Save before switching to {}", args.name))?;

    load_command_inner(ctx, args)
}
