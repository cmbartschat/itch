use crate::{
    cli::SaveArgs, ctx::Ctx, error::Attempt, remote::push_branch, reset::reset_repo, save::save,
};

pub fn save_command(ctx: &Ctx, args: &SaveArgs, silent: bool) -> Attempt {
    save(ctx, args, silent)?;
    let branch_name = ctx
        .repo
        .branches(Some(git2::BranchType::Local))?
        .find(|c| c.as_ref().is_ok_and(|b| b.0.is_head()));

    match push_branch(
        ctx,
        branch_name.unwrap().unwrap().0.name().unwrap().unwrap(),
    ) {
        Err(e) => println!("Skipping remote push due to: {}", e.message()),
        _ => {}
    }

    reset_repo(&ctx)?;
    Ok(())
}
