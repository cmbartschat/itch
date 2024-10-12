use crate::{
    branch::get_current_branch, cli::SaveArgs, ctx::Ctx, error::Attempt, remote::try_push_branch,
    reset::reset_repo, save::save,
};

pub fn save_command(ctx: &Ctx, args: &SaveArgs, silent: bool) -> Attempt {
    save(ctx, args, silent)?;
    let branch_name = get_current_branch(ctx)?;

    try_push_branch(ctx, &branch_name);

    reset_repo(ctx)?;
    Ok(())
}
