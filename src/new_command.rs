use git2::Error;
use log::debug;

use crate::{
    branch::choose_random_branch_name,
    cli::{LoadArgs, NewArgs},
    ctx::Ctx,
    load_command::load_command,
};

pub fn new_command(ctx: &Ctx, args: &NewArgs) -> Result<(), Error> {
    let name = match &args.name {
        Some(n) => Ok(n.to_string()),
        None => choose_random_branch_name(&ctx),
    }?;

    debug!("Creating new branch: {} from main", name);

    let base_branch = ctx.repo.find_branch("main", git2::BranchType::Local)?;

    let base_commit = base_branch.get().peel_to_commit()?;

    ctx.repo.branch(&name, &base_commit, false)?;

    load_command(&ctx, &LoadArgs { target: name })?;

    Ok(())
}
