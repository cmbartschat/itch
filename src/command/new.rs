use crate::{
    branch::choose_random_branch_name,
    cli::{LoadArgs, NewArgs},
    command::load::load_command,
    ctx::Ctx,
    error::Attempt,
};

pub fn new_command(ctx: &Ctx, args: &NewArgs) -> Attempt {
    let name = match &args.name {
        Some(n) => {
            if n.is_empty() {
                choose_random_branch_name(ctx)
            } else {
                Ok(n.to_string())
            }
        }
        None => choose_random_branch_name(ctx),
    }?;

    let base_branch = ctx.repo.find_branch("main", git2::BranchType::Local)?;

    let base_commit = base_branch.get().peel_to_commit()?;

    ctx.repo.branch(&name, &base_commit, false)?;

    if base_branch.is_head() {
        ctx.repo.set_head(&format!("refs/heads/{name}"))?;
    } else {
        load_command(ctx, &LoadArgs { name })?;
    }

    Ok(())
}
