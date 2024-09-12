use crate::{
    branch::choose_random_branch_name,
    cli::{LoadArgs, SplitArgs},
    ctx::Ctx,
    error::Attempt,
    save::save_temp,
};

use super::load::load_command;

pub fn split_command(ctx: &Ctx, args: &SplitArgs) -> Attempt {
    save_temp(ctx)?;

    let name: String = match &args.name {
        Some(n) => {
            if !n.is_empty() {
                Ok(n.to_string())
            } else {
                choose_random_branch_name(ctx)
            }
        }
        None => choose_random_branch_name(ctx),
    }?;

    let head_commit = ctx.repo.head()?.peel_to_commit()?;

    ctx.repo.branch(&name, &head_commit, false)?;

    load_command(ctx, &LoadArgs { name: name.clone() })?;

    if ctx.can_prompt() {
        eprintln!("Split to {name}");
    }

    Ok(())
}
