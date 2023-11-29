use git2::Error;
use log::debug;

use crate::{
    cli::{LoadArgs, SaveArgs},
    ctx::Ctx,
    save_command::save_command,
};

pub fn _load_command(ctx: &Ctx, args: &LoadArgs) -> Result<(), Error> {
    debug!("You want me to switch to: {}", args.target);

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

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Result<(), ()> {
    let message_vec = vec!["Save before switching to".to_string(), args.target.clone()];
    save_command(
        ctx,
        &SaveArgs {
            message: message_vec,
        },
    )?;

    return _load_command(ctx, args).map_err(|e| {
        debug!("Load failed: {:?}", e);
        ()
    });
}
