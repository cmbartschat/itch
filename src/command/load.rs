use git2::build::CheckoutBuilder;

use crate::{
    cli::{LoadArgs, SaveArgs},
    consts::TEMP_COMMIT_PREFIX,
    ctx::Ctx,
    error::{fail, Attempt},
    reset::pop_and_reset,
    save::save,
    timer::Timer,
};

pub fn _load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    let target_ref = ctx
        .repo
        .find_branch(&args.name, git2::BranchType::Local)?
        .into_reference();

    if let Some(target) = target_ref.name() {
        ctx.repo.set_head(target)?;
        let mut options = CheckoutBuilder::new();
        options.force();
        ctx.repo.checkout_head(Some(&mut options.into()))?;

        pop_and_reset(ctx)?;
        Ok(())
    } else {
        fail("Invalid branch name")
    }
}

pub fn load_command(ctx: &Ctx, args: &LoadArgs) -> Attempt {
    let mut timer = Timer::new("load_command");
    let message_vec = vec![
        TEMP_COMMIT_PREFIX.to_string(),
        "Save before switching to".to_string(),
        args.name.clone(),
    ];
    save(
        ctx,
        &SaveArgs {
            message: message_vec,
        },
        true,
    )?;

    timer.step("something");

    timer.step("after save");

    _load_command(ctx, args)?;

    timer.step("after load");

    timer.done();

    Ok(())
}
