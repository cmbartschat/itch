use anyhow::bail;

use crate::{
    cli::ConnectArgs,
    ctx::Ctx,
    error::Attempt,
    prompt::ask_option,
    remote::{connect_remote, pull_main, reset_main_to_remote},
    reset::pop_and_reset,
    save::save_temp,
};

pub fn connect_command(ctx: &Ctx, args: &ConnectArgs) -> Attempt {
    save_temp(ctx, "Save before connect".to_string())?;

    connect_remote(ctx, &args.url)?;
    todo!();

    // ctx.repo
    //     .branch_remote_tracking_ref_name("main", "origin/main");

    if pull_main(ctx).is_err() {
        if !ctx.can_prompt() {
            bail!("Added remote, but local main branch has diverged from origin.");
        }

        let ignore_option = "ignore";
        let reset_option = "reset local to main";

        let options = [ignore_option, reset_option];

        let chosen_option = ask_option(
            "Conflicts detected between local main and remote main.",
            &options,
            None,
        );

        if chosen_option == reset_option {
            reset_main_to_remote(ctx)?;
        }
    }

    pop_and_reset(ctx)?;
    Ok(())
}
