use crate::{
    cli::SaveArgs, ctx::Ctx, error::Attempt, remote::push_branch, reset::reset_repo, save::save,
    timer::Timer,
};

pub fn save_command(ctx: &Ctx, args: &SaveArgs, silent: bool) -> Attempt {
    let mut timer = Timer::new("save_command");
    save(ctx, args, silent)?;
    timer.step("saved");
    let branch_name = ctx
        .repo
        .branches(Some(git2::BranchType::Local))?
        .find(|c| c.as_ref().is_ok_and(|b| b.0.is_head()));

    match push_branch(
        ctx,
        branch_name.unwrap().unwrap().0.name().unwrap().unwrap(),
    ) {
        Err(e) => eprintln!("Skipping remote push due to: {}", e.message()),
        _ => {}
    }

    timer.step("save remote");

    reset_repo(&ctx)?;
    timer.step("reset");
    Ok(())
}
