use log::debug;

use crate::{base::resolve_base, branch::choose_random_branch_name, ctx::Ctx};

pub struct NewCommandArgs {
    pub name: Option<String>,
    pub base: Option<String>,
}

pub fn new_command(ctx: &Ctx, args: NewCommandArgs) -> Result<(), ()> {
    debug!("Resolving base");
    let base = resolve_base(args.base);

    log::debug!("Doing new command with base: {:?}", base);

    let name = match args.name {
        Some(n) => Ok(n),
        None => choose_random_branch_name(ctx),
    }?;

    debug!("Creating new branch with name: {}", name);

    Ok(())
}
