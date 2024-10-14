use crate::{cli::DeleteArgs, ctx::Ctx, error::Attempt, remote::try_delete_remote_branch};

pub fn delete_command(ctx: &Ctx, args: &DeleteArgs) -> Attempt {
    for branch_name in &args.names {
        let mut branch = ctx.repo.find_branch(branch_name, git2::BranchType::Local)?;
        branch.delete()?;
        try_delete_remote_branch(ctx, branch_name);
    }

    Ok(())
}
