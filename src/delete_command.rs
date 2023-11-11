use log::debug;

use crate::{cli::DeleteArgs, ctx::Ctx};

pub fn delete_command(ctx: &Ctx, args: &DeleteArgs) -> Result<(), ()> {
    debug!("Deleting branches: {:?}", args.names);

    for branch in &args.names {
        let mut branch = ctx
            .repo
            .find_branch(&branch, git2::BranchType::Local)
            .expect("Unknown branch");
        branch.delete().expect("Failed to delete");
    }

    Ok(())
}
