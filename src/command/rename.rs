use crate::{
    branch::get_current_branch,
    cli::RenameArgs,
    ctx::Ctx,
    error::{Attempt, fail},
    remote::try_delete_remote_branch,
};

pub fn rename_command(ctx: &Ctx, args: &RenameArgs) -> Attempt {
    let old_name = get_current_branch(ctx)?;
    if old_name == "main" {
        return fail("Cannot rename the main branch");
    }
    let ref_name = format!("refs/heads/{}", args.name);
    let message = format!("Renaming current branch to {}", args.name);
    ctx.repo.head()?.rename(&ref_name, false, &message)?;
    try_delete_remote_branch(ctx, &old_name);
    if ctx.can_prompt() {
        eprintln!("Renamed to {}", args.name);
    }
    Ok(())
}
