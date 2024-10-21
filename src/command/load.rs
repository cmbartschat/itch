use crate::{
    cli::LoadArgs,
    ctx::Ctx,
    error::{fail, Attempt},
    reset::pop_and_reset,
    save::save_temp,
};

pub fn _load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    let target_ref = ctx
        .repo
        .find_branch(&args.name, git2::BranchType::Local)?
        .into_reference();

    if let Some(target) = target_ref.name() {
        ctx.repo.set_head(target)?;
        ctx.repo.reset(
            target_ref.peel_to_commit()?.as_object(),
            git2::ResetType::Hard,
            None,
        )?;
        pop_and_reset(ctx)?;
        Ok(())
    } else {
        fail("Invalid branch name")
    }
}

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    save_temp(ctx, format!("Save before switching to {}", args.name))?;

    _load_command(ctx, args)
}
