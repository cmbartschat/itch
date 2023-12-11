use git2::Error;

use crate::{
    cli::{LoadArgs, SaveArgs},
    command::save::save_command,
    ctx::Ctx,
};

pub fn _load_command(ctx: &Ctx, args: &LoadArgs) -> Result<(), Error> {
    let target_ref = ctx
        .repo
        .find_branch(&args.target, git2::BranchType::Local)?
        .into_reference();

    if let Some(target) = target_ref.name() {
        ctx.repo.set_head(target)?;
        ctx.repo.reset(
            target_ref.peel_to_commit()?.as_object(),
            git2::ResetType::Hard,
            None,
        )?;
        Ok(())
    } else {
        Err(Error::from_str("Invalid branch name"))
    }
}

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Result<(), Error> {
    let message_vec = vec!["Save before switching to".to_string(), args.target.clone()];
    save_command(
        ctx,
        &SaveArgs {
            message: message_vec,
        },
        true,
    )?;

    _load_command(ctx, args)
}
