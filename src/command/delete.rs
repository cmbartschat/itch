use crate::{
    branch::find_branch,
    cli::{DeleteArgs, LoadArgs},
    ctx::Ctx,
    error::Attempt,
    remote::try_delete_remote_branch,
};

use super::load::load_command;

pub fn delete_command(ctx: &Ctx, args: &DeleteArgs) -> Attempt {
    let head_name = ctx.repo.head_ref()?.map(|f| f.name().to_owned());
    for branch_name in &args.names {
        let branch = find_branch(ctx, branch_name)?;
        if head_name
            .as_ref()
            .is_some_and(|f| f == &branch.name().to_owned())
        {
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
