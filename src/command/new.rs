use crate::{
    branch::{choose_random_branch_name, find_main},
    cli::{LoadArgs, NewArgs},
    command::load::load_command,
    ctx::Ctx,
    error::Attempt,
};

pub fn new_command(ctx: &Ctx, args: &NewArgs) -> Attempt {
    let name = match &args.name {
        Some(n) => {
            if n.is_empty() {
                choose_random_branch_name(ctx)
            } else {
                Ok(n.to_string())
            }
        }
        None => choose_random_branch_name(ctx),
    }?;

    let mut base_branch = find_main(ctx)?;

    let base_commit = base_branch.peel_to_id()?;

    ctx.repo.reference(
        name,
        base_commit,
        gix::refs::transaction::PreviousValue::MustNotExist,
        "",
    )?;

    todo!();

    load_command(ctx, &LoadArgs { name })?;

    Ok(())
}
