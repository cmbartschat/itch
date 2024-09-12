use crate::{cli::DeleteArgs, ctx::Ctx, error::Attempt};

pub fn delete_command(ctx: &Ctx, args: &DeleteArgs) -> Attempt {
    for branch in &args.names {
        let mut branch = ctx.repo.find_branch(branch, git2::BranchType::Local)?;
        branch.delete()?;
    }

    Ok(())
}
