use git2::Error;

use crate::{cli::DeleteArgs, ctx::Ctx};

pub fn delete_command(ctx: &Ctx, args: &DeleteArgs) -> Result<(), Error> {
    for branch in &args.names {
        let mut branch = ctx.repo.find_branch(&branch, git2::BranchType::Local)?;
        branch.delete()?;
    }

    Ok(())
}
