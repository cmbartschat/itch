use crate::{
    cli::{DeleteArgs, LoadArgs},
    ctx::Ctx,
    error::Attempt,
    remote::try_delete_remote_branch,
};

use super::load::load_command;

pub fn delete_command(ctx: &Ctx, args: &DeleteArgs) -> Attempt {
    for branch_name in &args.names {
        let mut branch = ctx.repo.find_branch(branch_name, git2::BranchType::Local)?;
        if branch.is_head() {
            load_command(
                ctx,
                &LoadArgs {
                    name: "main".to_string(),
                },
            )?;
        }
        branch.delete()?;
        try_delete_remote_branch(ctx, branch_name);
    }

    Ok(())
}
