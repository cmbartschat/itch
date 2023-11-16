use git2::Error;
use log::debug;

use crate::{
    cli::{LoadArgs, SaveArgs},
    ctx::Ctx,
    save_command::save_command,
};

pub fn _load_command(ctx: &Ctx, args: &LoadArgs) -> Result<(), Error> {
    debug!("You want me to switch to: {}", args.target);

    let branch = ctx
        .repo
        .find_branch(&args.target, git2::BranchType::Local)?;

    let object = branch.into_reference().peel_to_commit()?.into_object();

    ctx.repo.reset(&object, git2::ResetType::Hard, None)?;

    Ok(())
}

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Result<(), ()> {
    save_command(ctx, &SaveArgs { message: vec![] })?;

    return _load_command(ctx, args).map_err(|e| {
        debug!("Load failed: {:?}", e);
        ()
    });
}
