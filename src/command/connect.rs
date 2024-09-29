use crate::{
    cli::ConnectArgs,
    ctx::Ctx,
    error::{fail, Attempt},
    prompt::ask_option,
    remote::{connect_remote, pull_main, reset_main_to_remote},
    reset::pop_and_reset,
    save::save_temp,
};

pub fn connect_command(ctx: &Ctx, args: &ConnectArgs) -> Attempt {
    save_temp(ctx)?;

    println!("you want me to connect to {}", args.url);
    connect_remote(ctx, &args.url)?;

    let mut main_branch = ctx.repo.find_branch("main", git2::BranchType::Local)?;
    main_branch.set_upstream(Some("origin/main"))?;

    match pull_main(ctx) {
        Ok(()) => {}
        Err(_) => {
            if !ctx.can_prompt() {
                return fail("Added remote, but local main branch has diverged from origin.");
            }

            let options = ["ignore", "reset"];
            match ask_option("Failed to pull from origin.", &options, None).as_str() {
                "ignore" => {}
                "reset" => {
                    reset_main_to_remote(ctx)?;
                }
                _ => panic!("Unhandled option"),
            }
        }
    }

    pop_and_reset(ctx)?;
    Ok(())
}
