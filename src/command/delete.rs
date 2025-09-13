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

        match branch.delete() {
            Ok(()) => {}
            Err(e) => {
                // Due to multivars in config breaking libgit2, we retry if config cleanup fails
                // https://github.com/libgit2/libgit2/issues/6722
                if e.class() == git2::ErrorClass::Config {
                    branch.delete()?;
                } else {
                    return Err(e);
                }
            }
        }
        try_delete_remote_branch(ctx, branch_name);
    }

    Ok(())
}
