use log::debug;

use crate::{base::resolve_base, branch::choose_random_branch_name, cli::NewArgs, ctx::Ctx};

pub fn new_command(ctx: &Ctx, args: &NewArgs) -> Result<(), ()> {
    debug!("Resolving base");
    let base = resolve_base(&args.base)?;

    let name = match &args.name {
        Some(n) => Ok(n.to_string()),
        None => choose_random_branch_name(&ctx),
    }?;

    debug!("Creating new branch: {} from base: {}", name, base);

    let base_branch = ctx
        .repo
        .find_branch(&base, git2::BranchType::Local)
        .map_err(|_e| ())?;

    let base_commit = base_branch.get().peel_to_commit().map_err(|_e| ())?;

    ctx.repo
        .branch(&name, &base_commit, false)
        .map_err(|_e| ())?;

    Ok(())
}
