use crate::{
    cli::{LoadArgs, SaveArgs},
    command::save::save_command,
    consts::TEMP_COMMIT_PREFIX,
    ctx::Ctx,
    error::{fail, Attempt},
    reset::pop_and_reset,
};

pub fn _load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    let target_ref = ctx
        .repo
        .find_branch(&args.name, git2::BranchType::Local)?
        .into_reference();

    if let Some(target) = target_ref.name() {
        ctx.repo.set_head(target)?;
        ctx.repo.reset(
            target_ref.peel_to_commit()?.as_object(),
            git2::ResetType::Hard,
            None,
        )?;
        pop_and_reset(ctx)?;
        Ok(())
    } else {
        fail("Invalid branch name")
    }
}

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    let message_vec = vec![
        TEMP_COMMIT_PREFIX.to_string(),
        "Save before switching to".to_string(),
        args.name.clone(),
    ];
    save_command(
        ctx,
        &SaveArgs {
            message: message_vec,
        },
        true,
    )?;

    _load_command(ctx, args)
}
